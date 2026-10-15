//! The unused-definitions linter: every reference definition is used.

use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct UnusedDefinitions;

impl DocumentLinter for UnusedDefinitions {
    fn about(&self) -> About {
        About {
            name: "unused-definitions",
            description: "every reference definition is used by a reference link",
            since: FIRST_RELEASE,
            example: Example::new(
                "See the [guide].\n\n[guide]: docs/guide.md\n[old]: docs/old.md\n",
                "See the [guide].\n\n[guide]: docs/guide.md\n",
            )
            .with_files(&[EXAMPLE_GUIDE]),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let mut out: Vec<Finding> = document
            .definitions
            .iter()
            .filter(|(label, _)| !document.used_labels.contains(*label))
            .map(|(label, def)| Finding::new(path, def.line, format!("unused definition: [{label}]")))
            .collect();
        out.sort_by_key(|issue| issue.line);
        Ok(out)
    }
}
