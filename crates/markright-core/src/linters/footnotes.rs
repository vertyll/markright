//! The footnotes linter: every footnote a document refers to is defined, once, and every one it defines is referred
//! to.

use std::collections::{HashMap, HashSet};

use super::{About, Context, DocumentLinter, Example};
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

pub(super) struct Footnotes;

impl DocumentLinter for Footnotes {
    fn about(&self) -> About {
        About {
            name: "footnotes",
            description: "a footnote is defined once and referred to; one defined nowhere renders as the text `[^label]`",
            since: "v0.3.0",
            example: Example::new(
                "Kafka keeps the order[^order] within a partition[^partition].\n\n[^order]: Of the records.\n[^unused]: Never cited.\n",
                "Kafka keeps the order[^order] within a partition.\n\n[^order]: Of the records.\n",
            ),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let mut out: Vec<Finding> = document
            .undefined_footnotes
            .iter()
            .map(|reference| {
                Finding::new(
                    path,
                    reference.line,
                    format!("undefined footnote: [^{}]", reference.label),
                )
            })
            .collect();
        let mut first_defined: HashMap<&str, usize> = HashMap::new();
        for definition in &document.footnote_definitions {
            match first_defined.get(definition.label.as_str()) {
                Some(first) => out.push(Finding::new(
                    path,
                    definition.line,
                    format!("[^{}] is already defined on line {first}", definition.label),
                )),
                None => {
                    first_defined.insert(&definition.label, definition.line);
                }
            }
        }
        let used: HashSet<&str> = document
            .footnote_references
            .iter()
            .map(|reference| reference.label.as_str())
            .collect();
        out.extend(
            document
                .footnote_definitions
                .iter()
                .filter(|definition| first_defined.get(definition.label.as_str()) == Some(&definition.line))
                .filter(|definition| !used.contains(definition.label.as_str()))
                .map(|definition| {
                    Finding::new(
                        path,
                        definition.line,
                        format!("unused footnote: [^{}]", definition.label),
                    )
                }),
        );
        out.sort_by_key(|finding| finding.line);
        Ok(out)
    }
}
