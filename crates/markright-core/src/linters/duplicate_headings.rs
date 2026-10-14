//! The duplicate-headings linter: no two headings of one section say the same.

use std::collections::HashSet;

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct DuplicateHeadings;

impl DocumentLinter for DuplicateHeadings {
	fn about(&self) -> About {
		About {
			name: "duplicate-headings",
			description: "no two headings of one section say the same, which would leave their anchors apart by a number only",
			since: FIRST_RELEASE,
			example: Example::new(
				"# Guide\n\n## Setup\n\n## Usage\n\n## Setup\n",
				"# Guide\n\n## Setup\n\n## Usage\n\n## Upgrade\n",
			),
		}
	}

	/// Headings are siblings when the same heading is the nearest one above them of a lower level.
	fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		let mut seen: HashSet<(Option<usize>, u8, String)> = HashSet::new();
		let mut found = Vec::new();
		for (index, heading) in document.headings.iter().enumerate() {
			let parent = document.headings[..index]
				.iter()
				.rposition(|above| above.level < heading.level);
			let key = (parent, heading.level, heading.text.to_lowercase());
			if !seen.insert(key) {
				let message = format!("heading repeats one of its section: {}", heading.text);
				found.push(Finding::new(path, heading.line, message));
			}
		}
		Ok(found)
	}
}
