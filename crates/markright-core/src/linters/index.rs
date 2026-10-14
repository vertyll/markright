//! The index linter: the index document links every document of the repository.

use super::{About, Context, EXAMPLE_GUIDE, Example, INDEX_DOCUMENT, RepositoryLinter};
use crate::FIRST_RELEASE;
use crate::issue::Finding;

pub(super) struct Index;

impl RepositoryLinter for Index {
	fn about(&self) -> About {
		About {
			name: "index",
			description: "the index document links every Markdown document of the repository",
			since: FIRST_RELEASE,
			example: Example::new(
				"# Contents\n\n- [Guide](docs/guide.md)\n",
				"# Contents\n\n- [Guide](docs/guide.md)\n- [Setup](docs/setup.md)\n",
			)
			.at(INDEX_DOCUMENT)
			.with_files(&[(EXAMPLE_GUIDE.0, "# Guide\n"), ("docs/setup.md", "# Setup\n")]),
		}
	}

	fn check(&self, context: &Context<'_>) -> Vec<Finding> {
		let file = &context.settings.index.file;
		if !context.repo.docs.contains_key(file) {
			return Vec::new();
		}
		let linked = context.repo.linked(file);
		let missing = context
			.repo
			.docs
			.keys()
			.filter(|&path| path != file && !linked.contains(path) && !(context.excluded)(path));
		missing
			.map(|path| Finding::new(file.clone(), 1, format!("document not listed: {path}")))
			.collect()
	}
}
