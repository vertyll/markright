use super::{About, Context, EXAMPLE_GUIDE, Example, INDEX_DOCUMENT, RepositoryLinter, V0_1_0};
use crate::issue::Finding;

pub(super) struct Index;

impl RepositoryLinter for Index {
	fn about(&self) -> About {
		About {
			name: "index",
			description: "the index document links every Markdown document of the repository",
			since: V0_1_0,
			example: Example::new(
				"# Contents\n\n- [Guide](docs/guide.md)\n",
				"# Contents\n\n- [Guide](docs/guide.md)\n- [Setup](docs/setup.md)\n",
			)
			.at(INDEX_DOCUMENT)
			.with_files(&[(EXAMPLE_GUIDE.0, "# Guide\n"), ("docs/setup.md", "# Setup\n")]),
		}
	}

	fn check(&self, c: &Context<'_>) -> Vec<Finding> {
		let file = &c.settings.index.file;
		if !c.repo.docs.contains_key(file) {
			return Vec::new();
		}
		let linked = c.repo.linked(file);
		let missing = c
			.repo
			.docs
			.keys()
			.filter(|&p| p != file && !linked.contains(p) && !(c.excluded)(p));
		missing
			.map(|p| Finding::new(file.clone(), 1, format!("document not listed: {p}")))
			.collect()
	}
}
