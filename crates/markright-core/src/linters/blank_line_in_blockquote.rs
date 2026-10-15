//! The blank-line-in-blockquote linter: a blank line does not split what is meant as one blockquote into two.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{BlockKind, Document};
use crate::{Error, FIRST_RELEASE};

pub(super) struct BlankLineInBlockquote;

impl DocumentLinter for BlankLineInBlockquote {
    fn about(&self) -> About {
        About {
            name: "blank-line-in-blockquote",
            description: "a blank line does not split a blockquote in two; a `>` alone keeps its paragraphs together",
            since: FIRST_RELEASE,
            example: Example::new(
                "> The first paragraph of the quote.\n\n> The second one.\n",
                "> The first paragraph of the quote.\n>\n> The second one.\n",
            ),
        }
    }

    /// Two alerts in a row are two blockquotes on purpose, so a quote that starts with an alert tag is its own.
    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let quotes: Vec<_> = document
            .blocks
            .iter()
            .filter(|block| block.kind == BlockKind::BlockQuote && block.depth == 0)
            .collect();
        let mut found = Vec::new();
        for pair in quotes.windows(2) {
            let (first, second) = (pair[0], pair[1]);
            let only_blank = (first.last + 1..second.first).all(|line| document.is_blank(line));
            let alert = document.lines[second.first - 1]
                .text
                .trim_start_matches(['>', ' '])
                .starts_with("[!");
            if second.first > first.last + 1 && only_blank && !alert {
                found.push(Finding::new(
                    path,
                    first.last + 1,
                    "blank line splits a blockquote in two",
                ));
            }
        }
        Ok(found)
    }
}
