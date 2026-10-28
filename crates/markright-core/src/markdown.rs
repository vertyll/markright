//! Parses a document with a `CommonMark` and GFM parser and keeps what the formatters and linters work with, by line:
//! which lines are code, the anchors the document defines, its headings, code blocks, list markers, tables, links,
//! reference definitions and the reference links nothing defines, the lazy continuation lines of its blockquotes, and
//! the byte ranges formatters must leave alone.

mod anchors;
mod destination;
mod document;
mod lines;
mod spaced_emphasis;
mod tables;
mod walker;

use pulldown_cmark::{Event, Parser};

pub use document::{
    Block, BlockKind, Definition, Document, Emphasis, Fence, Heading, ItemMarker, Line, Link, LinkKind, ListItem,
    Marker, Reference, Span, Table,
};
pub(crate) use tables::{is_table_row, table_cells};
#[cfg(test)]
pub(crate) use walker::OPTIONS;

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

/// What a document says, as the parser reads it: its blocks, inlines and text, whatever way they are written. A
/// formatter checks a change against it when the grammar has cases its own reading of a line cannot see, such as runs
/// of `*` and `_` that join, or a line below a list item that the item's indent decides about.
pub(crate) struct Reading {
    events: Vec<Event<'static>>,
}

impl Reading {
    /// What `text` says.
    pub(crate) fn of(text: &str) -> Self {
        Self {
            events: Parser::new_ext(text, walker::OPTIONS).map(Event::into_static).collect(),
        }
    }

    /// Whether `text` says the same, though it may be written differently.
    pub(crate) fn is(&self, text: &str) -> bool {
        let said: Vec<Event<'_>> = Parser::new_ext(text, walker::OPTIONS).collect();
        said.len() == self.events.len() && said.iter().zip(&self.events).all(|(said, read)| said == read)
    }
}

/// Whether the byte of `source` at `offset` is escaped: an odd number of backslashes stands right before it, so the
/// last of them escapes it rather than being escaped itself.
pub(crate) fn is_escaped(source: &str, offset: usize) -> bool {
    source[..offset].bytes().rev().take_while(|&byte| byte == b'\\').count() % 2 == 1
}

/// The full case foldings that lowercasing does not make, for the letters a Latin or Greek label may hold: `CommonMark`
/// matches labels by Unicode case folding, as the parser does, so `[ẞ]` finds `[SS]: /url`.
const FOLDINGS: &[(char, &str)] = &[
    ('ß', "ss"),
    ('ς', "σ"),
    ('ſ', "s"),
    ('ﬀ', "ff"),
    ('ﬁ', "fi"),
    ('ﬂ', "fl"),
    ('ﬃ', "ffi"),
    ('ﬄ', "ffl"),
    ('ﬅ', "st"),
    ('ﬆ', "st"),
];

/// Case-folds a reference or footnote label and collapses its runs of white space, as matching labels does.
pub(crate) fn normalize_label(label: &str) -> String {
    let lowercase = label.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    lowercase
        .chars()
        .fold(String::with_capacity(lowercase.len()), |mut folded, c| {
            match FOLDINGS.iter().find(|(from, _)| *from == c) {
                Some((_, to)) => folded.push_str(to),
                None => folded.push(c),
            }
            folded
        })
}

#[cfg(test)]
mod tests;
