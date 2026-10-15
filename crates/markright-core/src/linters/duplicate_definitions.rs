//! The duplicate-definitions linter: a label is defined once.

use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct DuplicateDefinitions;

impl DocumentLinter for DuplicateDefinitions {
    fn about(&self) -> About {
        About {
            name: "duplicate-definitions",
            description: "a reference label is defined once; a renderer ignores every definition after the first",
            since: FIRST_RELEASE,
            example: Example::new(
                "See the [guide].\n\n[guide]: docs/guide.md\n[guide]: docs/old.md\n",
                "See the [guide].\n\n[guide]: docs/guide.md\n",
            )
            .with_files(&[EXAMPLE_GUIDE]),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        Ok(document
            .duplicate_definitions
            .iter()
            .map(|duplicate| {
                let first = document
                    .definitions
                    .get(&duplicate.label)
                    .map_or(0, |definition| definition.line);
                let message = format!("[{}] is already defined on line {first}", duplicate.label);
                Finding::new(path, duplicate.line, message)
            })
            .collect())
    }
}
