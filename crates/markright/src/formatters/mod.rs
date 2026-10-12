//! The registry of formatters: rules that rewrite the prose lines of a document.

mod arrows;
mod blank_lines;
mod final_newline;
mod list_marker;
pub mod settings;
mod tables;
mod trailing_whitespace;
mod width;

use crate::markdown::{Document, Line};
pub use settings::Settings;

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

/// What a formatter made of a document: its new lines, and the 1-based numbers of the lines it changed or removed,
/// in order, or the last one when it adds lines at the end.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Fix {
	/// The lines of the formatted document.
	pub lines: Vec<String>,
	/// The lines of the input that changed.
	pub changed: Vec<usize>,
}

/// Every formatter, in the order they run. Those that remove or add lines come last.
pub static ALL: &[&dyn Formatter] = &[
	&trailing_whitespace::TrailingWhitespace,
	&arrows::Arrows,
	&list_marker::ListMarker,
	&tables::Tables,
	&blank_lines::BlankLines,
	&final_newline::FinalNewline,
];

/// The formatter called `name`.
#[must_use]
pub fn find(name: &str) -> Option<&'static dyn Formatter> {
	ALL.iter().copied().find(|f| f.about().name == name)
}

/// Rewrites every line that is not code with `rewrite`.
fn each_prose_line(d: &Document, rewrite: impl Fn(&Line) -> String) -> Fix {
	by_line(
		&d.lines,
		d.lines
			.iter()
			.map(|line| if line.code { line.text.clone() } else { rewrite(line) })
			.collect(),
	)
}

/// The text of every line of `d`, to rewrite some of them.
fn texts(d: &Document) -> Vec<String> {
	d.lines.iter().map(|line| line.text.clone()).collect()
}

/// The fix that turns the lines `before` into `after`, which has as many lines.
fn by_line(before: &[Line], after: Vec<String>) -> Fix {
	let changed = before
		.iter()
		.zip(&after)
		.enumerate()
		.filter(|(_, (b, a))| b.text != **a)
		.map(|(i, _)| i + 1)
		.collect();
	Fix { lines: after, changed }
}

#[cfg(test)]
mod tests;
