//! The registry of formatters: rules that rewrite the prose lines of a document.

mod arrows;
mod blank_lines;
mod final_newline;
mod fix;
mod list_marker;
mod rule;
pub mod settings;
mod tables;
mod trailing_whitespace;
mod width;

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

/// The formatter called `name`.
#[must_use]
pub fn find(name: &str) -> Option<&'static dyn Formatter> {
	ALL.iter().copied().find(|f| f.about().name == name)
}

#[cfg(test)]
mod tests;
