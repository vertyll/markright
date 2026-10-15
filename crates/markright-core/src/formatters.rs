//! The registry of formatters: rules that rewrite the prose lines of a document.

// What the formatters share.
mod fix;
mod rule;
pub mod settings;
mod width;

// The formatters, one per file.
mod arrows;
mod bare_urls;
mod blank_lines;
mod block_spacing;
mod blockquote_space;
mod code_span_spacing;
mod emphasis_marker;
mod fence_marker;
mod final_newline;
mod hard_tabs;
mod heading_spacing;
mod heading_style;
mod horizontal_rule;
mod link_spacing;
mod list_marker;
mod list_marker_space;
mod ordered_list_numbers;
mod table_pipes;
mod tables;
mod trailing_whitespace;

pub use fix::Fix;
pub use rule::{About, Formatter};
pub use settings::Settings;

use fix::{by_line, each_prose_line, texts};

/// Every formatter, in the order they run. Tabs go first, so the others see spaces; those that remove or add lines
/// come last.
pub static ALL: &[&dyn Formatter] = &[
    &hard_tabs::HardTabs,
    &trailing_whitespace::TrailingWhitespace,
    &heading_spacing::HeadingSpacing,
    &blockquote_space::BlockquoteSpace,
    &list_marker_space::ListMarkerSpace,
    &arrows::Arrows,
    &list_marker::ListMarker,
    &ordered_list_numbers::OrderedListNumbers,
    &emphasis_marker::EmphasisMarker,
    &link_spacing::LinkSpacing,
    &code_span_spacing::CodeSpanSpacing,
    &bare_urls::BareUrls,
    &fence_marker::FenceMarker,
    &horizontal_rule::HorizontalRule,
    &table_pipes::TablePipes,
    &tables::Tables,
    &heading_style::HeadingStyle,
    &block_spacing::BlockSpacing,
    &blank_lines::BlankLines,
    &final_newline::FinalNewline,
];

/// The name of every formatter, in the order they run.
#[must_use]
pub fn names() -> Vec<&'static str> {
    ALL.iter().map(|formatter| formatter.about().name).collect()
}

/// The formatter called `name`.
#[must_use]
pub fn find(name: &str) -> Option<&'static dyn Formatter> {
    ALL.iter().copied().find(|formatter| formatter.about().name == name)
}

#[cfg(test)]
mod tests;
