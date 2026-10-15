//! The horizontal-rule formatter: horizontal rules written one way, `---` or the style the settings choose.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::{BlockKind, Document};

pub(super) struct HorizontalRule;

impl Formatter for HorizontalRule {
    fn about(&self) -> About {
        About {
            name: "horizontal-rule",
            description: "writes a horizontal rule as `---`, or the style its settings choose",
            message: "horizontal rule written differently from the configured style",
            since: FIRST_RELEASE,
            example: "Before.\n\n* * *\n\nAfter.\n",
        }
    }

    /// With the style `---`, a rule right under a line of text stays as written, since there `---` underlines the text
    /// into a heading; and so does a rule on the first line, since there `---` opens front matter.
    fn apply(&self, document: &Document, settings: &Settings) -> Fix {
        let style = settings.horizontal_rule.style.as_str();
        let mut out = texts(document);
        let rules = document
            .blocks
            .iter()
            .filter(|block| block.kind == BlockKind::Rule && block.depth == 0);
        for rule in rules {
            let index = rule.first - 1;
            let misread = style.starts_with('-') && (rule.first == 1 || !document.is_blank(rule.first - 1));
            if out[index] != style && !misread {
                style.clone_into(&mut out[index]);
            }
        }
        by_line(&document.lines, out)
    }
}
