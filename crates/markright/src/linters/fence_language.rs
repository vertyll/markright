//! The fence-language linter: every fenced code block names its language.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct FenceLanguage;

impl DocumentLinter for FenceLanguage {
	fn about(&self) -> About {
		About {
			name: "fence-language",
			description: "a fenced code block names its language",
			since: FIRST_RELEASE,
			example: Example::new("```\nmake build\n```\n", "```bash\nmake build\n```\n"),
		}
	}

	fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		let bare = document.fences.iter().filter(|fence| fence.info.is_empty());
		Ok(bare
			.map(|fence| Finding::new(path, fence.line, "code block without a language"))
			.collect())
	}
}
