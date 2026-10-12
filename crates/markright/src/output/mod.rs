//! Writes issues in the formats `.markright.yaml` asks for: text, Checkstyle and SARIF.

mod checkstyle;
mod sarif;
mod text;

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

pub use text::text;

use crate::config::{Destination, Formats};
use crate::issue::Issue;

/// A formatter or linter as a report lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
	/// Its name.
	pub name: &'static str,
	/// What it checks, as text.
	pub description: String,
}

/// The writers behind the `stdout` and `stderr` destinations.
pub struct Streams<'a> {
	/// The standard output.
	pub stdout: &'a mut dyn Write,
	/// The standard error.
	pub stderr: &'a mut dyn Write,
}

/// Why a report could not be written.
#[derive(Debug, thiserror::Error)]
#[error("writing the {format} report{}: {source}", path.as_ref().map(|p| format!(" to {}", p.display())).unwrap_or_default())]
pub struct OutputError {
	/// The format.
	pub format: &'static str,
	/// The file, when it went to one.
	pub path: Option<PathBuf>,
	/// Why.
	pub source: io::Error,
}

type Writer = fn(&mut dyn Write, &[Issue], &[Rule]) -> io::Result<()>;

/// Writes `issues` in every format `formats` sets.
///
/// # Errors
///
/// Returns the first report that could not be written.
pub fn write(
	formats: &Formats,
	issues: &[Issue],
	rules: &[Rule],
	streams: &mut Streams<'_>,
) -> Result<(), OutputError> {
	let text_only = Formats::default();
	let formats = if formats.is_empty() { &text_only } else { formats };
	let writers: [(&'static str, _, Writer); 3] = [
		("text", &formats.text, |w, issues, _| text::write(w, issues)),
		("checkstyle", &formats.checkstyle, checkstyle::write),
		("sarif", &formats.sarif, sarif::write),
	];
	for (format, destination, write) in writers {
		let Some(destination) = destination else { continue };
		let error = |path: Option<PathBuf>| move |source| OutputError { format, path, source };
		match &destination.path {
			Destination::Stdout => write(streams.stdout, issues, rules).map_err(error(None))?,
			Destination::Stderr => write(streams.stderr, issues, rules).map_err(error(None))?,
			Destination::File(path) => {
				let to_file = || -> io::Result<()> {
					let mut file = BufWriter::new(File::create(path)?);
					write(&mut file, issues, rules)?;
					file.into_inner().map_err(io::IntoInnerError::into_error)?.sync_all()
				};
				to_file().map_err(error(Some(path.clone())))?;
			}
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests;
