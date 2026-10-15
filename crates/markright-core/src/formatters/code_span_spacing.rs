//! The code-span-spacing formatter: no padding spaces just inside the backticks of a code span.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct CodeSpanSpacing;

impl Formatter for CodeSpanSpacing {
    fn about(&self) -> About {
        About {
            name: "code-span-spacing",
            description: "removes the space just inside each backtick of a code span, which renders as nothing",
            message: "space inside the backticks of a code span",
            since: FIRST_RELEASE,
            example: "Run ` markright check ` in CI.\n",
        }
    }

    fn apply(&self, document: &Document, _: &Settings) -> Fix {
        let mut out = texts(document);
        // From the last span to the first, so the bytes of those before it stay where they are.
        for span in document.code_spans.iter().rev() {
            let index = span.line - 1;
            if let Some(tight) = tighten(&out[index][span.columns.clone()]) {
                out[index].replace_range(span.columns.clone(), &tight);
            }
        }
        by_line(&document.lines, out)
    }
}

/// The code span `span`, backticks included, without the space on each side of its code, or nothing when it has
/// none to lose. `CommonMark` drops one space from each side of code that has one on both, so only that pair goes,
/// and only when what is left neither starts nor ends with a space or a backtick, which the pair keeps as they are.
/// A space on one side only is part of the code, as in ` 9.`, and stays.
fn tighten(span: &str) -> Option<String> {
    let ticks = span.bytes().take_while(|&b| b == b'`').count();
    let (delimiter, code) = (&span[..ticks], &span[ticks..span.len() - ticks]);
    let inner = code.strip_prefix(' ')?.strip_suffix(' ')?;
    let kept = |c: char| c == ' ' || c == '`';
    let bare = !inner.is_empty() && !inner.starts_with(kept) && !inner.ends_with(kept);
    bare.then(|| format!("{delimiter}{inner}{delimiter}"))
}
