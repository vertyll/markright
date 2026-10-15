//! What a formatter makes of a document, and the ways the formatters make it.

use crate::markdown::{Document, Line};

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
