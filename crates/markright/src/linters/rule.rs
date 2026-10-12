//! What a linter is: what it says about itself, and the two kinds of linter the registry holds.

use std::fmt;

use super::Context;
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

/// What a linter is called and what it enforces, as the commands, the reports and the documentation show it.
#[derive(Debug, Clone, Copy)]
pub struct About {
	/// The name the configuration and the issues use.
	pub name: &'static str,
	/// One line of Markdown whose only markup is code spans; [`crate::markdown::plain`] gives it as text.
	pub description: &'static str,
	/// The release that added the linter.
	pub since: &'static str,
	/// A document the linter reports, and the same document fixed.
	pub example: Example,
}

/// A linter that looks at one document at a time.
pub(crate) trait DocumentLinter: Sync {
	/// What the linter is called and what it enforces.
	fn about(&self) -> About;

	/// What the linter finds in the document at `path`.
	///
	/// # Errors
	///
	/// Returns why the repository could not answer what the linter asked, such as whether a target exists.
	fn check(&self, context: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error>;
}

/// A linter that looks at the whole repository once, after every document is parsed.
pub(crate) trait RepositoryLinter: Sync {
	/// What the linter is called and what it enforces.
	fn about(&self) -> About;

	/// What the linter finds in the repository.
	fn check(&self, context: &Context<'_>) -> Vec<Finding>;
}

/// A linter of the registry. What it says about itself is open to anyone; running it is the runner's work.
#[derive(Clone, Copy)]
pub struct Linter(pub(crate) Kind);

/// The kind of a linter: of one document at a time, or of the whole repository.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
	/// Runs once per document.
	Document(&'static dyn DocumentLinter),
	/// Runs once for the whole repository.
	Repository(&'static dyn RepositoryLinter),
}

impl Linter {
	pub(super) const fn document(linter: &'static dyn DocumentLinter) -> Self {
		Self(Kind::Document(linter))
	}

	pub(super) const fn repository(linter: &'static dyn RepositoryLinter) -> Self {
		Self(Kind::Repository(linter))
	}

	/// What the linter is called and what it enforces.
	#[must_use]
	pub fn about(self) -> About {
		match self.0 {
			Kind::Document(linter) => linter.about(),
			Kind::Repository(linter) => linter.about(),
		}
	}
}

impl fmt::Debug for Linter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Linter").field(&self.about().name).finish()
	}
}

/// A linter at work: `bad` is a document it reports, `good` the same document fixed. `path` is where the document
/// lives, and `files` are the other files of the repository the linter looks at.
#[derive(Debug, Clone, Copy)]
pub struct Example {
	/// A document the linter reports.
	pub bad: &'static str,
	/// The document fixed.
	pub good: &'static str,
	/// Where the document lives.
	pub path: &'static str,
	/// The other files of the repository, by path.
	pub files: &'static [(&'static str, &'static str)],
}

impl Example {
	pub(super) const fn new(bad: &'static str, good: &'static str) -> Self {
		Self {
			bad,
			good,
			path: "doc.md",
			files: &[],
		}
	}

	pub(super) const fn with_files(self, files: &'static [(&'static str, &'static str)]) -> Self {
		Self { files, ..self }
	}

	pub(super) const fn at(self, path: &'static str) -> Self {
		Self { path, ..self }
	}
}
