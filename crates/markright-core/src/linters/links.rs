//! The links linter: relative links lead to files inside the repository, and every link has a target and text.

use super::settings::targeted;
use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example, escapes, targets};
use crate::issue::Finding;
use crate::markdown::{Document, LinkKind};
use crate::{Error, FIRST_RELEASE};

pub(super) struct Links;

impl DocumentLinter for Links {
    fn about(&self) -> About {
        About {
            name: "links",
            description: "a relative link points to a file or directory inside the repository",
            since: FIRST_RELEASE,
            example: Example::new("See the [guide](docs/gide.md).\n", "See the [guide](docs/guide.md).\n")
                .with_files(&[EXAMPLE_GUIDE]),
        }
    }

    fn check(&self, context: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let mut out = Vec::new();
        for target in targets(path, document) {
            let Some(file) = &target.file else { continue };
            if !escapes(file) && !targeted(&context.settings.links.targets, file) {
                continue;
            }
            if escapes(file) {
                out.push(Finding::new(
                    path,
                    target.line,
                    format!("link leaves the repository: {}", target.raw),
                ));
            } else if !context.repo.exists(file).map_err(|source| Error::Resolve {
                linter: self.about().name,
                path: file.clone(),
                source,
            })? {
                out.push(Finding::new(path, target.line, format!("broken link: {}", target.raw)));
            }
        }
        for link in &document.links {
            match &link.kind {
                LinkKind::Inline { text, .. } | LinkKind::Image { alt: text } if link.target.is_empty() => {
                    out.push(Finding::new(
                        path,
                        link.line,
                        format!("link without a target: [{text}]()"),
                    ));
                }
                LinkKind::Inline { has_content: false, .. } => {
                    out.push(Finding::new(
                        path,
                        link.line,
                        format!("link without text: []({})", link.target),
                    ));
                }
                _ => {}
            }
        }
        for reference in &document.undefined_references {
            out.push(Finding::new(
                path,
                reference.line,
                format!("undefined reference: [{}]", reference.label),
            ));
        }
        Ok(out)
    }
}
