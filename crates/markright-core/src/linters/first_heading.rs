//! The first-heading linter: a document starts with its title, a top-level heading.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{BlockKind, Document};
use crate::{Error, FIRST_RELEASE};

pub(super) struct FirstHeading;

impl DocumentLinter for FirstHeading {
    fn about(&self) -> About {
        About {
            name: "first-heading",
            description: "a document starts with a top-level heading, after any front matter and HTML",
            since: FIRST_RELEASE,
            example: Example::new("Some notes.\n\n# Guide\n", "# Guide\n\nSome notes.\n"),
        }
    }

    /// Front matter and HTML, such as badges or a logo, may come before the heading.
    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let first = document
            .blocks
            .iter()
            .find(|block| block.depth == 0 && !matches!(block.kind, BlockKind::Metadata | BlockKind::Html));
        let Some(first) = first else { return Ok(Vec::new()) };
        let title = first.kind == BlockKind::Heading
            && document
                .headings
                .iter()
                .any(|heading| heading.line == first.first && heading.level == 1);
        Ok(if title {
            Vec::new()
        } else {
            vec![Finding::new(
                path,
                first.first,
                "document does not start with a top-level heading",
            )]
        })
    }
}
