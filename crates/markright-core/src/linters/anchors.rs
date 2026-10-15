//! The anchors linter: a `#fragment` names a heading or an HTML id of its document.

use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example, escapes, targets};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct Anchors;

impl DocumentLinter for Anchors {
    fn about(&self) -> About {
        About {
            name: "anchors",
            description: "a `#fragment` matches a heading of the target document",
            since: FIRST_RELEASE,
            example: Example::new(
                "See [installing](docs/guide.md#setup).\n",
                "See [installing](docs/guide.md#install).\n",
            )
            .with_files(&[EXAMPLE_GUIDE]),
        }
    }

    fn check(&self, context: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let broken = targets(path, document).into_iter().filter(|target| {
            let Some(anchor) = &target.anchor else { return false };
            let file = target.document(path);
            !escapes(file)
                && context
                    .repo
                    .docs
                    .get(file)
                    .is_some_and(|linked| !linked.anchors.contains(&anchor.to_lowercase()))
        });
        Ok(broken
            .map(|target| Finding::new(path, target.line, format!("broken anchor: {}", target.raw)))
            .collect())
    }
}
