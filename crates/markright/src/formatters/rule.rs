//! What a formatter is: what it says about itself, and how it formats.

use super::{Fix, Settings};
use crate::markdown::Document;

/// What a formatter is called and what it does, as the commands, the reports and the documentation show it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct About {
	/// The name the configuration and the issues use.
	pub name: &'static str,
	/// One line of Markdown whose only markup is code spans; [`crate::markdown::plain`] gives it as text.
	pub description: &'static str,
	/// The message of an issue for a line the formatter would change.
	pub message: &'static str,
	/// The release that added the formatter.
	pub since: &'static str,
	/// A document the formatter changes; what it makes of it shows the fix.
	pub example: &'static str,
}

/// A rule that rewrites the prose of a document.
pub trait Formatter: Sync {
	/// What the formatter is called and what it does.
	fn about(&self) -> About;

	/// Formats the parsed document, which says which lines are code and where its lists, tables and protected ranges
	/// are, with the settings of the formatters, of which it reads its own. It leaves the document as it is.
	fn apply(&self, document: &Document, settings: &Settings) -> Fix;
}
