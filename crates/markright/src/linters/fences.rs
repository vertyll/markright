use super::{About, Context, DocumentLinter, Example, V0_1_0};
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

pub(super) struct Fences;

impl DocumentLinter for Fences {
	fn about(&self) -> About {
		About {
			name: "fences",
			description: "every code block is closed",
			since: V0_1_0,
			example: Example::new("```bash\nmake build\n", "```bash\nmake build\n```\n"),
		}
	}

	fn check(&self, _: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		Ok(doc
			.unclosed_fence
			.map(|line| Finding::new(path, line, "code block is never closed"))
			.into_iter()
			.collect())
	}
}
