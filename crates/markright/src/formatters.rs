//! The registry of formatters: rules that rewrite the prose lines of a document.

// What the formatters share.
mod fix;
mod rule;
pub mod settings;
mod width;

// The formatters, one per file.
mod arrows;
mod blank_lines;
mod final_newline;
mod list_marker;
mod tables;
mod trailing_whitespace;

pub use fix::Fix;
pub use rule::{About, Formatter};
pub use settings::Settings;

use fix::{by_line, each_prose_line, texts};

/// Every formatter, in the order they run. Those that remove or add lines come last.
pub static ALL: &[&dyn Formatter] = &[
	&trailing_whitespace::TrailingWhitespace,
	&arrows::Arrows,
	&list_marker::ListMarker,
	&tables::Tables,
	&blank_lines::BlankLines,
	&final_newline::FinalNewline,
];

/// The name of every formatter, in the order they run.
#[must_use]
pub fn names() -> Vec<&'static str> {
	ALL.iter().map(|f| f.about().name).collect()
}

/// The formatter called `name`.
#[must_use]
pub fn find(name: &str) -> Option<&'static dyn Formatter> {
	ALL.iter().copied().find(|f| f.about().name == name)
}

#[cfg(test)]
mod tests;
