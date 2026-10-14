//! The emphasis-as-heading linter: a line of bold or italic text alone is not used in place of a heading.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{BlockKind, Document};
use crate::{Error, FIRST_RELEASE};

/// The characters that end a sentence, which a heading does not end with.
const SENTENCE_END: [char; 6] = ['.', ',', ';', ':', '!', '?'];

pub(super) struct EmphasisAsHeading;

impl DocumentLinter for EmphasisAsHeading {
	fn about(&self) -> About {
		About {
			name: "emphasis-as-heading",
			description: "a paragraph of emphasis alone is not used in place of a heading",
			since: FIRST_RELEASE,
			example: Example::new(
				"# Guide\n\n**Install**\n\nRun the installer.\n",
				"# Guide\n\n## Install\n\nRun the installer.\n",
			),
		}
	}

	/// A paragraph of one line, at the top of the document, that is all emphasis and does not end like a sentence.
	fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		let paragraphs = document
			.blocks
			.iter()
			.filter(|block| block.kind == BlockKind::Paragraph && block.depth == 0 && block.first == block.last);
		let mut found = Vec::new();
		for paragraph in paragraphs {
			let text = &document.lines[paragraph.first - 1].text;
			let start = text.len() - text.trim_start().len();
			let end = text.trim_end().len();
			let whole = document.emphasis.iter().any(|emphasis| {
				emphasis.open.line == paragraph.first
					&& emphasis.close.line == paragraph.first
					&& emphasis.open.columns.start == start
					&& emphasis.close.columns.end == end
			});
			let inner = text[start..end].trim_matches(['*', '_']);
			if whole && !inner.ends_with(SENTENCE_END) {
				found.push(Finding::new(path, paragraph.first, "emphasis used as a heading"));
			}
		}
		Ok(found)
	}
}
