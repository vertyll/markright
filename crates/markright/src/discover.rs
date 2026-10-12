//! Lists the Markdown documents of a repository: what Git tracks or would add, minus build and dependency
//! directories.

use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::Error;
use crate::fs::is_markdown;

/// The directories no document is taken from, wherever they are.
const EXCLUDED_DIRS: &[&str] = &[
	".git",
	".gradle",
	".idea",
	".kotlin",
	".next",
	".nuxt",
	".output",
	".scannerwork",
	".terraform",
	".turbo",
	"build",
	"coverage",
	"dist",
	"dokka",
	"node_modules",
	"target",
	"vendor",
];

/// The Markdown documents under `root`, relative to it, with forward slashes, sorted. It asks Git, and walks the
/// directory when Git cannot answer; it stops once `stop` is set.
///
/// # Errors
///
/// Returns [`Error::Stopped`] when stopped, or why the directory could not be walked.
pub(crate) fn markdown(root: &Path, stop: &AtomicBool) -> Result<Vec<String>, Error> {
	let stopped = || stop.load(Ordering::Relaxed);
	if stopped() {
		return Err(Error::Stopped);
	}
	let listed = git_listed(root).map_or_else(|| walked(root, stop), Ok)?;
	if stopped() {
		return Err(Error::Stopped);
	}
	let mut out: Vec<String> = listed.into_iter().filter(|p| !in_excluded_dir(p)).collect();
	out.sort_unstable();
	Ok(out)
}

/// What Git tracks or would add, and still exists; None when Git cannot answer, outside a repository or without Git.
fn git_listed(root: &Path) -> Option<Vec<String>> {
	let output = Command::new("git")
		.args([
			"ls-files",
			"--cached",
			"--others",
			"--exclude-standard",
			"-z",
			"--",
			"*.md",
		])
		.current_dir(root)
		.output()
		.ok()
		.filter(|output| output.status.success())?;
	let listed = String::from_utf8(output.stdout).ok()?;
	Some(
		listed
			.split('\0')
			.filter(|p| !p.is_empty() && root.join(p).exists())
			.map(str::to_owned)
			.collect(),
	)
}

fn walked(root: &Path, stop: &AtomicBool) -> Result<Vec<String>, Error> {
	let mut out = Vec::new();
	walk(root, "", stop, &mut out)?;
	Ok(out)
}

fn walk(dir: &Path, prefix: &str, stop: &AtomicBool, out: &mut Vec<String>) -> Result<(), Error> {
	let error = |source: io::Error| Error::Walk {
		path: dir.to_owned(),
		source,
	};
	for entry in dir.read_dir().map_err(error)? {
		if stop.load(Ordering::Relaxed) {
			return Err(Error::Stopped);
		}
		let entry = entry.map_err(error)?;
		let name = entry.file_name().to_string_lossy().into_owned();
		let path = format!("{prefix}{name}");
		let kind = entry.file_type().map_err(error)?;
		if kind.is_dir() {
			if !EXCLUDED_DIRS.contains(&name.as_str()) {
				walk(&entry.path(), &format!("{path}/"), stop, out)?;
			}
		} else if is_markdown(&path) {
			out.push(path);
		}
	}
	Ok(())
}

fn in_excluded_dir(path: &str) -> bool {
	path.rsplit_once('/')
		.is_some_and(|(dir, _)| dir.split('/').any(|part| EXCLUDED_DIRS.contains(&part)))
}

#[cfg(test)]
mod tests {
	use std::fs;
	use std::process::Command;
	use std::sync::atomic::AtomicBool;

	use super::markdown;
	use crate::Error;

	fn tree(names: &[&str]) -> tempfile::TempDir {
		let root = tempfile::tempdir().unwrap();
		for name in names {
			let path = root.path().join(name);
			fs::create_dir_all(path.parent().unwrap()).unwrap();
			fs::write(path, "x").unwrap();
		}
		root
	}

	#[test]
	fn walks_outside_git() {
		let root = tree(&[
			"README.md",
			"docs/a.md",
			"docs/a.txt",
			"node_modules/x/README.md",
			"src/build/b.md",
		]);
		assert_eq!(
			markdown(root.path(), &AtomicBool::new(false)).unwrap(),
			["README.md", "docs/a.md"]
		);
	}

	#[test]
	fn git_honors_its_ignores_and_takes_untracked_files() {
		let root = tree(&[".gitignore", "README.md", "docs/a.md", "out/ignored.md", "target/b.md"]);
		fs::write(root.path().join(".gitignore"), "out/\n").unwrap();
		let Ok(init) = Command::new("git")
			.args(["init", "-q"])
			.current_dir(root.path())
			.status()
		else {
			return; // no Git here
		};
		assert!(init.success());
		assert_eq!(
			markdown(root.path(), &AtomicBool::new(false)).unwrap(),
			["README.md", "docs/a.md"]
		);
	}

	#[test]
	fn listing_stops_when_asked() {
		let root = tree(&["a.md"]);
		assert!(matches!(
			markdown(root.path(), &AtomicBool::new(true)),
			Err(Error::Stopped)
		));
	}
}
