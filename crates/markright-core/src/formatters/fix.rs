//! What a formatter makes of a document, and the ways the formatters make it.

use std::cmp::Reverse;
use std::ops::Range;

use crate::markdown::{Document, Line, Reading};

/// What a formatter made of a document: its new lines, and the 1-based numbers of the lines it changed or removed,
/// in order, or the last one when it adds lines at the end.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Fix {
    /// The lines of the formatted document.
    pub lines: Vec<String>,
    /// The lines of the input that changed.
    pub changed: Vec<usize>,
}

/// Rewrites every line that is not code with `rewrite`.
pub(super) fn each_prose_line(document: &Document, rewrite: impl Fn(&Line) -> String) -> Fix {
    by_line(
        &document.lines,
        document
            .lines
            .iter()
            .map(|line| if line.code { line.text.clone() } else { rewrite(line) })
            .collect(),
    )
}

/// The text of every line of `document`, to rewrite some of them.
pub(super) fn texts(document: &Document) -> Vec<String> {
    document.lines.iter().map(|line| line.text.clone()).collect()
}

/// The fix that turns the lines `before` into `after`, which has as many lines.
pub(super) fn by_line(before: &[Line], after: Vec<String>) -> Fix {
    let changed = before
        .iter()
        .zip(&after)
        .enumerate()
        .filter(|(_, (b, a))| b.text != **a)
        .map(|(i, _)| i + 1)
        .collect();
    Fix { lines: after, changed }
}

/// One replacement a formatter wants to make: bytes of one line, as the document has them, and what goes in their
/// place.
pub(super) struct Replacement {
    /// The 0-based index of the line.
    pub(super) line: usize,
    /// The bytes of the line, in the document as it is.
    pub(super) bytes: Range<usize>,
    /// What takes their place.
    pub(super) text: String,
}

/// A change a formatter wants to make, kept or dropped as a whole: one or more replacements, such as both delimiters
/// of one emphasis.
pub(super) type Edit = Vec<Replacement>;

/// Makes the `edits` to `document`, keeping only those after which it reads the same: all of them at once when they
/// keep what it says, which is the usual case, or else halves of them in turn, down to the single edits that change
/// it, so a few such edits in a long document cost a few parses each rather than one per edit. Every edit is made on
/// the lines as the document has them, so no edit moves the bytes another one names; replacements must not overlap.
pub(super) fn keeping_meaning(document: &Document, edits: &[Edit]) -> Fix {
    let lines = texts(document);
    let reading = Reading::of(&document.to_string());
    let mut kept = Vec::new();
    settle(&lines, &reading, &mut kept, edits);
    by_line(&document.lines, with(&lines, &kept))
}

/// Adds to `kept` the `edits` that, with those already kept, leave the document reading as `reading` says: all of
/// them when they do, or else those each half of them keeps.
fn settle<'a>(lines: &[String], reading: &Reading, kept: &mut Vec<&'a Edit>, edits: &'a [Edit]) {
    if edits.is_empty() {
        return;
    }
    let tried: Vec<&Edit> = kept.iter().copied().chain(edits).collect();
    if reading.is(&with(lines, &tried).join("\n")) {
        *kept = tried;
    } else if edits.len() > 1 {
        let (first, second) = edits.split_at(edits.len() / 2);
        settle(lines, reading, kept, first);
        settle(lines, reading, kept, second);
    }
}

/// `lines` with the replacements of every one of `edits`, made from the last byte of each line back.
fn with(lines: &[String], edits: &[&Edit]) -> Vec<String> {
    let mut replacements: Vec<&Replacement> = edits.iter().flat_map(|edit| edit.iter()).collect();
    replacements.sort_by_key(|replacement| (replacement.line, Reverse(replacement.bytes.start)));
    let mut out = lines.to_vec();
    for replacement in replacements {
        out[replacement.line].replace_range(replacement.bytes.clone(), &replacement.text);
    }
    out
}
