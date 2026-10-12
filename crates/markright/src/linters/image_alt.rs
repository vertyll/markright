use super::{About, Context, DocumentLinter, Example, V0_1_0};
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

pub(super) struct ImageAlt;

impl DocumentLinter for ImageAlt {
	fn about(&self) -> About {
		About {
			name: "image-alt",
			description: "an image has alternative text",
			since: V0_1_0,
			example: Example::new(
				"![](flow.png)\n",
				"![Request flow from gateway to database](flow.png)\n",
			),
		}
	}

	fn check(&self, _: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		let bare = doc.links.iter().filter(|link| link.image && link.text.is_empty());
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
