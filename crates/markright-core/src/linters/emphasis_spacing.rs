//! The emphasis-spacing linter: emphasis has no space just inside its delimiters, which would leave it as text.

use super::{About, Context, DocumentLinter, Example};
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

pub(super) struct EmphasisSpacing;

impl DocumentLinter for EmphasisSpacing {
    fn about(&self) -> About {
        About {
            name: "emphasis-spacing",
            description: "emphasis has no space just inside its delimiters, which would render `** text **` as written",
            since: "v0.3.0",
            example: Example::new(
                "# Upgrade\n\nTake a backup ** before ** the upgrade.\n",
                "# Upgrade\n\nTake a backup **before** the upgrade.\n",
            ),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        Ok(document
            .spaced_emphasis
            .iter()
            .map(|span| {
                let written = &document.lines[span.line - 1].text[span.columns.clone()];
                Finding::new(
                    path,
                    span.line,
                    format!("space inside emphasis, which renders as written: {written}"),
                )
            })
            .collect())
    }
}
