//! Where each line of the source starts, to turn the parser's byte offsets into lines and columns.

use std::ops::{Range, RangeInclusive};

/// The byte offset each line of a text starts at.
pub(super) struct LineIndex {
    starts: Vec<usize>,
    len: usize,
}

impl LineIndex {
    /// The index of `text`.
    pub(super) fn new(text: &str) -> Self {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        Self {
            starts,
            len: text.len(),
        }
    }

    /// The 0-based line `offset` is on: the last line that starts at or before it.
    pub(super) fn line(&self, offset: usize) -> usize {
        self.starts.partition_point(|&start| start <= offset) - 1
    }

    /// The byte of its line `offset` is at.
    pub(super) fn column(&self, offset: usize) -> usize {
        offset - self.starts[self.line(offset)]
    }

    /// The text of the 0-based `line` of `source`, the text this index was made from.
    pub(super) fn text<'s>(&self, source: &'s str, line: usize) -> &'s str {
        &source[self.starts[line]..self.end(line)]
    }

    /// How many lines the text has.
    pub(super) fn count(&self) -> usize {
        self.starts.len()
    }

    /// Where the 0-based `line` ends, before its newline.
    pub(super) fn end(&self, line: usize) -> usize {
        self.starts.get(line + 1).map_or(self.len, |next| next - 1)
    }

    /// The 0-based lines `range` covers: from the line it starts on to the line of its last byte.
    pub(super) fn lines(&self, range: &Range<usize>) -> RangeInclusive<usize> {
        self.line(range.start)..=self.line(range.end.saturating_sub(1).max(range.start))
    }
}
