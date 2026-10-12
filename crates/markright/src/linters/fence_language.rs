use super::{About, Context, DocumentLinter, Example, V0_1_0};
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

pub(super) struct FenceLanguage;

impl DocumentLinter for FenceLanguage {
	fn about(&self) -> About {
		About {
			name: "fence-language",
			description: "a fenced code block names its language",
			since: V0_1_0,
			example: Example::new("```\nmake build\n```\n", "```bash\nmake build\n```\n"),
		}
	}

	fn check(&self, _: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		let bare = doc.fences.iter().filter(|fence| fence.info.is_empty());
		Ok(bare
			.map(|fence| Finding::new(path, fence.line, "code block without a language"))
			.collect())
	}
}
