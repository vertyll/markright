//! A problem at one line of a document: what a rule finds, and the issue a run reports, which names the rule.

use std::fmt;

/// What a rule found at one line of a document. The runner turns it into an [`Issue`] that names the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Finding {
	/// The document, relative to the repository root, with forward slashes.
	pub(crate) path: String,
	/// The 1-based line.
	pub(crate) line: usize,
	/// What is wrong.
	pub(crate) message: String,
}

impl Finding {
	/// A finding at `line` of the document at `path`.
	#[must_use]
	pub(crate) fn new(path: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
		Self {
			path: path.into(),
			line,
			message: message.into(),
		}
	}

	/// The issue this finding is, found by the formatter or linter called `source`.
	#[must_use]
	pub(crate) fn found_by(self, source: &'static str) -> Issue {
		Issue {
			path: self.path,
			line: self.line,
			message: self.message,
			source,
		}
	}
}

/// A problem at one line of a document, named by the formatter or linter that found it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Issue {
	/// The document, relative to the repository root, with forward slashes.
	pub path: String,
	/// The 1-based line.
	pub line: usize,
	/// What is wrong.
	pub message: String,
	/// The formatter or linter that found it.
	pub source: &'static str,
}

impl fmt::Display for Issue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}:{}: {} ({})", self.path, self.line, self.message, self.source)
	}
}

#[cfg(test)]
mod tests {
	use super::Finding;

	#[test]
	fn an_issue_names_its_rule() {
		let issue = Finding::new("docs/a.md", 4, "broken link: x.md").found_by("links");
		assert_eq!(issue.to_string(), "docs/a.md:4: broken link: x.md (links)");
	}
}
