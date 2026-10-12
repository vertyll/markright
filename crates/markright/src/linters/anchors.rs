use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example, escapes, targets};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct Anchors;

impl DocumentLinter for Anchors {
	fn about(&self) -> About {
		About {
			name: "anchors",
			description: "a `#fragment` matches a heading of the target document",
			since: FIRST_RELEASE,
			example: Example::new(
				"See [installing](docs/guide.md#setup).\n",
				"See [installing](docs/guide.md#install).\n",
			)
			.with_files(&[EXAMPLE_GUIDE]),
		}
	}

	fn check(&self, c: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		let broken = targets(path, doc).into_iter().filter(|t| {
			let Some(anchor) = &t.anchor else { return false };
			let file = t.document(path);
			!escapes(file)
				&& c.repo
					.docs
					.get(file)
					.is_some_and(|target| !target.anchors.contains(&anchor.to_lowercase()))
		});
		Ok(broken
			.map(|t| Finding::new(path, t.line, format!("broken anchor: {}", t.raw)))
			.collect())
	}
}
