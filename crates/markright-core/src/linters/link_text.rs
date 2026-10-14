//! The link-text linter: a link's text says where it leads, not "here" or "click here".

use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example};
use crate::issue::Finding;
use crate::markdown::{Document, LinkKind};
use crate::{Error, FIRST_RELEASE};

pub(super) struct LinkText;

impl DocumentLinter for LinkText {
	fn about(&self) -> About {
		About {
			name: "link-text",
			description: "a link's text says where it leads, not one of the phrases its settings list, such as `here`",
			since: FIRST_RELEASE,
			example: Example::new(
				"The guide is [here](docs/guide.md).\n",
				"Read [the guide](docs/guide.md).\n",
			)
			.with_files(&[EXAMPLE_GUIDE]),
		}
	}

	fn check(&self, context: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		let phrases = &context.settings.link_text.phrases;
		let mut found = Vec::new();
		for link in &document.links {
			let LinkKind::Inline { text, .. } = &link.kind else {
				continue;
			};
			let bare = text
				.trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
				.to_lowercase();
			if phrases.iter().any(|phrase| phrase.to_lowercase() == bare) {
				found.push(Finding::new(
					path,
					link.line,
					format!("link text says nothing about its target: {text}"),
				));
			}
		}
		Ok(found)
	}
}
