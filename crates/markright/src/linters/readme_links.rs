use super::{About, Context, Example, INDEX_DOCUMENT, RepositoryLinter, V0_1_0};
use crate::issue::Finding;

pub(super) struct ReadmeLinks;

impl RepositoryLinter for ReadmeLinks {
	fn about(&self) -> About {
		About {
			name: "readme-links",
			description: "the README links each of the configured documents that exists",
			since: V0_1_0,
			example: Example::new(
				"# Project\n\nA tool.\n",
				"# Project\n\nA tool. See the [contents](CONTENTS.md).\n",
			)
			.at("README.md")
			.with_files(&[(INDEX_DOCUMENT, "# Contents\n\n- [Project](README.md)\n")]),
		}
	}

	fn check(&self, c: &Context<'_>) -> Vec<Finding> {
		let settings = &c.settings.readme_links;
		if !c.repo.docs.contains_key(&settings.file) {
			return Vec::new();
		}
		let linked = c.repo.linked(&settings.file);
		let missing = settings
			.links
			.iter()
			.filter(|want| c.repo.docs.contains_key(*want) && !linked.contains(*want));
		missing
			.map(|want| Finding::new(settings.file.clone(), 1, format!("does not link {want}")))
			.collect()
	}
}
