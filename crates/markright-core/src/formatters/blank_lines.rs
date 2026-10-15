//! The blank-lines formatter: at most one blank line in a row outside code.

use super::{About, Fix, Formatter, Settings};
use crate::FIRST_RELEASE;
use crate::markdown::{Document, Line};

pub(super) struct BlankLines;

impl Formatter for BlankLines {
    fn about(&self) -> About {
        About {
            name: "blank-lines",
            description: "leaves at most one blank line in a row outside code",
            message: "more than one blank line in a row",
            since: FIRST_RELEASE,
            example: "First paragraph.\n\n\n\nSecond paragraph.\n",
        }
    }

    fn apply(&self, document: &Document, _: &Settings) -> Fix {
        let blank = |line: &Line| !line.code && line.text.trim().is_empty();
        let end = content_end(&document.lines);
        let mut fix = Fix {
            lines: Vec::with_capacity(document.lines.len()),
            changed: Vec::new(),
        };
        for (i, line) in document.lines.iter().enumerate() {
            if i > 0 && i < end && blank(line) && blank(&document.lines[i - 1]) {
                fix.changed.push(i + 1);
            } else {
                fix.lines.push(line.text.clone());
            }
        }
        fix
    }
}

/// How many lines there are up to the last one that is not blank.
pub(super) fn content_end(lines: &[Line]) -> usize {
    lines
        .iter()
        .rposition(|line| !line.text.trim().is_empty())
        .map_or(0, |i| i + 1)
}
