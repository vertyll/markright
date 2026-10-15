//! The image-alt linter: every image has alternative text.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{Document, LinkKind};
use crate::{Error, FIRST_RELEASE};

pub(super) struct ImageAlt;

impl DocumentLinter for ImageAlt {
    fn about(&self) -> About {
        About {
            name: "image-alt",
            description: "an image has alternative text",
            since: FIRST_RELEASE,
            example: Example::new(
                "![](flow.png)\n",
                "![Request flow from gateway to database](flow.png)\n",
            ),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let bare = document
            .links
            .iter()
            .filter(|link| matches!(&link.kind, LinkKind::Image { alt } if alt.is_empty()));
        Ok(bare
            .map(|link| {
                Finding::new(
                    path,
                    link.line,
                    format!("image without alternative text: {}", link.target),
                )
            })
            .collect())
    }
}
