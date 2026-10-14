//! The heading-punctuation linter: a heading does not end like a sentence.

use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct HeadingPunctuation;

/// The punctuation a heading does not end with; `?` may, as a question names a section of answers.
const PUNCTUATION: [char; 10] = ['.', ',', ';', ':', '!', '。', '，', '；', '：', '！'];

impl DocumentLinter for HeadingPunctuation {
	fn about(&self) -> About {
		About {
			name: "heading-punctuation",
			description: "a heading does not end with `.`, `,`, `;`, `:` or `!`",
			since: FIRST_RELEASE,
			example: Example::new("# Guide\n\n## Install:\n", EXAMPLE_GUIDE.1),
		}
	}

	fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		Ok(document
			.headings
			.iter()
			.filter_map(|heading| {
				let last = heading
					.text
					.trim_end()
					.chars()
					.last()
					.filter(|c| PUNCTUATION.contains(c))?;
				Some(Finding::new(path, heading.line, format!("heading ends with {last}")))
			})
			.collect())
	}
}
