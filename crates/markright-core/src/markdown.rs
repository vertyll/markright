//! Parses a document with a `CommonMark` and GFM parser and keeps what the formatters and linters work with, by line:
//! which lines are code, the anchors the document defines, its headings, code blocks, list markers, tables, links,
//! reference definitions and the reference links nothing defines, the lazy continuation lines of its blockquotes, and
//! the byte ranges formatters must leave alone.

mod anchors;
mod destination;
mod document;
mod lines;
mod tables;
mod walker;

pub use document::{
    Block, BlockKind, Definition, Document, Emphasis, Fence, Heading, ItemMarker, Line, Link, LinkKind, ListItem,
    Marker, Reference, Span, Table,
};
pub(crate) use tables::{is_table_row, table_cells};

/// Whether a link target leaves the repository: a URL with a scheme, or a mail address.
#[must_use]
pub(crate) fn is_external(target: &str) -> bool {
    target.contains("://") || target.starts_with("mailto:")
}

/// A line of Markdown whose only markup is code spans, as text: the code spans lose their backticks.
#[must_use]
pub fn plain(markdown: &str) -> String {
    markdown.replace('`', "")
}

/// Lowercases a reference label and collapses its runs of white space, as matching labels does.
pub(crate) fn normalize_label(label: &str) -> String {
    label.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

#[cfg(test)]
mod tests;
