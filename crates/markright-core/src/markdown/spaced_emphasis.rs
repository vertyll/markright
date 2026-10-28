//! Finds emphasis written with a space just inside its delimiters, `** text **`, which `CommonMark` does not read as
//! emphasis: an opening run must not be followed by white space, nor a closing one preceded by it, so the markers
//! render as they are written.

use std::ops::Range;

use pulldown_cmark::{Event, Parser, Tag};

use super::is_escaped;

/// The most delimiters a run of emphasis has: three, for strong emphasis inside emphasis.
const LONGEST_RUN: usize = 3;

/// A run of `*` or `_`: where it is and its character.
struct Run {
    range: Range<usize>,
    marker: char,
}

/// The stretches of `source` within `prose`, one line of text the parser left as text, that are emphasis with a space
/// just inside a delimiter: a run of `*` or `_` and a run of the same, around text with a letter in it, of which the
/// parser would make emphasis without the spaces. A run escaped with `\`, or inside a word, marks nothing; nor does a
/// lone `*` with white space on both sides of it and of its partner, which is how a product is written, `a * b * c`.
/// The ranges are offsets into `source`.
pub(super) fn find(source: &str, prose: &Range<usize>) -> Vec<Range<usize>> {
    let runs = runs(source, prose);
    let mut found = Vec::new();
    let mut next = 0;
    while next < runs.len() {
        let open = &runs[next];
        next += 1;
        if !opens(source, &open.range) {
            continue;
        }
        let close = runs[next..]
            .iter()
            .position(|close| close.marker == open.marker && close.range.len() == open.range.len())
            .map(|offset| next + offset);
        let Some(close_index) = close else { continue };
        let close = &runs[close_index];
        let inner = &source[open.range.end..close.range.start];
        let spaced = inner.starts_with(char::is_whitespace) || inner.ends_with(char::is_whitespace);
        let content = inner.trim();
        let product = open.marker == '*'
            && open.range.len() == 1
            && spaced_around(source, &open.range)
            && spaced_around(source, &close.range);
        if spaced
            && !product
            && closes(source, &close.range)
            && content.chars().any(char::is_alphabetic)
            && is_emphasis(&source[open.range.clone()], content)
        {
            found.push(open.range.start..close.range.end);
            next = close_index + 1;
        }
    }
    found
}

/// The runs of one to three `*` or `_` in `prose`, unescaped, in order.
fn runs(source: &str, prose: &Range<usize>) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut chars = source[prose.clone()].char_indices().peekable();
    while let Some((offset, c)) = chars.next() {
        if c != '*' && c != '_' {
            continue;
        }
        let start = prose.start + offset;
        let mut end = start + 1;
        while chars.next_if(|&(_, next)| next == c).is_some() {
            end += 1;
        }
        if end - start <= LONGEST_RUN && !is_escaped(source, start) {
            runs.push(Run {
                range: start..end,
                marker: c,
            });
        }
    }
    runs
}

/// Whether the run at `range` may open emphasis: at the start of the line, or after white space or a character that
/// opens something, `(`, `[`, `"` or `'`, rather than inside a word or a path such as `src/*`.
fn opens(source: &str, range: &Range<usize>) -> bool {
    source[..range.start]
        .chars()
        .next_back()
        .is_none_or(|before| before.is_whitespace() || "([\"'".contains(before))
}

/// Whether the run at `range` may close emphasis: at the end of the line, or before white space or punctuation.
fn closes(source: &str, range: &Range<usize>) -> bool {
    source[range.end..]
        .chars()
        .next()
        .is_none_or(|after| after.is_whitespace() || after.is_ascii_punctuation())
}

/// Whether white space stands on both sides of the run at `range`.
fn spaced_around(source: &str, range: &Range<usize>) -> bool {
    let before = source[..range.start].chars().next_back();
    let after = source[range.end..].chars().next();
    before.is_some_and(char::is_whitespace) && after.is_some_and(char::is_whitespace)
}

/// Whether `content` between two runs written as `delimiters` is emphasis once nothing separates them.
fn is_emphasis(delimiters: &str, content: &str) -> bool {
    let joined = format!("{delimiters}{content}{delimiters}");
    let mut events = Parser::new(&joined);
    matches!(events.next(), Some(Event::Start(Tag::Paragraph)))
        && matches!(events.next(), Some(Event::Start(Tag::Emphasis | Tag::Strong)))
}
