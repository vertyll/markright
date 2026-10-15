//! The blockquote-space formatter: a space between the `>` of a blockquote and its text.

use super::{About, Fix, Formatter, Settings, each_prose_line};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct BlockquoteSpace;

impl Formatter for BlockquoteSpace {
    fn about(&self) -> About {
        About {
            name: "blockquote-space",
            description: "puts a space between the `>` of a blockquote and its text",
            message: "blockquote text right after its `>`",
            since: FIRST_RELEASE,
            example: ">Restart the service after a change.\n",
        }
    }

    fn apply(&self, document: &Document, _: &Settings) -> Fix {
        each_prose_line(document, |line| spaced(&line.text))
    }
}

/// `line` with a space after the `>` it starts with, each after up to three spaces of indentation, unless another `>`
/// follows it right away, as in `>>`.
fn spaced(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + 2);
    let mut rest = line;
    loop {
        let indent = rest.len() - rest.trim_start_matches(' ').len();
        let Some(after) = rest[indent..].strip_prefix('>').filter(|_| indent <= 3) else {
            break;
        };
        out.push_str(&rest[..indent]);
        out.push('>');
        if !after.is_empty() && !after.starts_with([' ', '\t', '>']) {
            out.push(' ');
        }
        rest = after;
        if let Some(spaced) = rest.strip_prefix(' ') {
            out.push(' ');
            rest = spaced;
        }
    }
    out.push_str(rest);
    out
}
