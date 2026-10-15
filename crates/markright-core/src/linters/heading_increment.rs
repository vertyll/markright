//! The heading-increment linter: headings go down one level at a time.

use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct HeadingIncrement;

impl DocumentLinter for HeadingIncrement {
    fn about(&self) -> About {
        About {
            name: "heading-increment",
            description: "a heading is at most one level deeper than the heading before it",
            since: FIRST_RELEASE,
            example: Example::new("# Guide\n\n### Install\n", EXAMPLE_GUIDE.1),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let jumps = document
            .headings
            .windows(2)
            .filter(|pair| pair[1].level > pair[0].level + 1);
        let message = |from: u8, to: u8| format!("heading level jumps from {from} to {to}");
        Ok(jumps
            .map(|pair| Finding::new(path, pair[1].line, message(pair[0].level, pair[1].level)))
            .collect())
    }
}
