//! The single-title linter: a document has one top-level heading, its title.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct SingleTitle;

impl DocumentLinter for SingleTitle {
    fn about(&self) -> About {
        About {
            name: "single-title",
            description: "a document has one top-level heading, its title",
            since: FIRST_RELEASE,
            example: Example::new("# Guide\n\n# Install\n", "# Guide\n\n## Install\n"),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        Ok(document
            .headings
            .iter()
            .filter(|heading| heading.level == 1)
            .skip(1)
            .map(|heading| Finding::new(path, heading.line, "a second top-level heading"))
            .collect())
    }
}
