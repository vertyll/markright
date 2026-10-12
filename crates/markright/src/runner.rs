//! Makes one pass over a repository: it formats every document, lints the result and filters the issues through the
//! exclusions. [`Runner`] does it on disk and writes the result; [`check_documents`] does it in memory.

#[cfg(feature = "disk")]
mod disk;
mod parallel;
mod pass;

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

#[cfg(feature = "disk")]
pub use disk::{Report, Runner};

pub(crate) use pass::check;

use crate::Error;
use crate::config::Config;
use crate::fs::is_markdown;
use crate::issue::Issue;

/// What a run does with the formatters' changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	/// Reports what the formatters would change and runs the linters.
	Lint,
	/// Keeps what the formatters change, and runs no linter.
	Format,
}

/// What a check found: the issues, sorted by path and line, the formatted content of every document the formatters
/// change, by path, and the warnings for watched exclusions that matched nothing.
#[derive(Debug, Default)]
pub struct Checked {
	/// The issues, sorted by path, line and message.
	pub issues: Vec<Issue>,
	/// The new content of each document the formatters change.
	pub fixed: BTreeMap<String, String>,
	/// A warning for each watched exclusion that matched nothing.
	pub unused_exclusions: Vec<String>,
}

/// Checks a repository held in memory: every file in `documents`, by path, is part of it, and the ones ending in
/// `.md` are the documents.
///
/// # Errors
///
/// Returns why the documents could not be checked.
pub fn check_documents(documents: &BTreeMap<String, String>, config: &Config, mode: Mode) -> Result<Checked, Error> {
	let paths: Vec<String> = documents.keys().filter(|p| is_markdown(p)).cloned().collect();
	check(documents, &paths, config, mode, &AtomicBool::new(false))
}

#[cfg(all(test, feature = "disk"))]
mod tests;

#[cfg(test)]
mod in_memory {
	use std::collections::BTreeMap;

	use super::{Mode, check_documents};
	use crate::config::Config;

	#[test]
	fn documents_in_memory() {
		let documents = BTreeMap::from([
			(
				"doc.md".to_owned(),
				"# Title\n\nGo -> [here](guide.md) and [there](img.png).\n".to_owned(),
			),
			("img.png".to_owned(), String::new()),
		]);
		let checked = check_documents(&documents, &Config::default(), Mode::Lint).unwrap();
		let issues: Vec<String> = checked.issues.iter().map(ToString::to_string).collect();
		assert_eq!(
			issues,
			[
				"doc.md:3: ASCII arrow or missing space around an arrow (arrows)",
				"doc.md:3: broken link: guide.md (links)"
			]
		);
		assert_eq!(
			checked.fixed["doc.md"],
			"# Title\n\nGo → [here](guide.md) and [there](img.png).\n"
		);
	}
}
