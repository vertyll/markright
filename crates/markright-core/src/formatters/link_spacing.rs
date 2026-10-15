//! The link-spacing formatter: no spaces just inside the brackets of a link's text.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct LinkSpacing;

impl Formatter for LinkSpacing {
    fn about(&self) -> About {
        About {
            name: "link-spacing",
            description: "removes the spaces just inside the brackets of a link's text",
            message: "space inside the brackets of a link",
            since: FIRST_RELEASE,
            example: "See the [ guide ](guide.md).\n",
        }
    }

    fn apply(&self, document: &Document, _: &Settings) -> Fix {
        let mut out = texts(document);
        for label in document.links.iter().filter_map(|link| link.label.as_ref()) {
            let index = label.line - 1;
            let text = &out[index][label.columns.clone()];
            let trimmed = text.trim();
            if !document.lines[index].code && !trimmed.is_empty() && trimmed.len() != text.len() {
                let trimmed = trimmed.to_owned();
                out[index].replace_range(label.columns.clone(), &trimmed);
            }
        }
        by_line(&document.lines, out)
    }
}
