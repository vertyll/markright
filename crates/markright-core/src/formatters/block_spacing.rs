//! The block-spacing formatter: a blank line before and after every heading, list, fenced code block and table.

use super::{About, Fix, Formatter, Settings};
use crate::FIRST_RELEASE;
use crate::markdown::{BlockKind, Document};

/// The blocks that stand apart from what is around them.
const SPACED: [BlockKind; 4] = [
	BlockKind::Heading,
	BlockKind::List,
	BlockKind::FencedCode,
	BlockKind::Table,
];

pub(super) struct BlockSpacing;

impl Formatter for BlockSpacing {
	fn about(&self) -> About {
		About {
			name: "block-spacing",
			description: "puts a blank line before and after every heading, list, fenced code block and table",
			message: "heading, list, code block or table without a blank line around it",
			since: FIRST_RELEASE,
			example: "# Install\nRun:\n```bash\nmake install\n```\n",
		}
	}

	/// Only blocks at the top of the document: inside a list or a blockquote, a blank line can change what the
	/// container holds.
	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		let spaced = document
			.blocks
			.iter()
			.filter(|block| block.depth == 0 && SPACED.contains(&block.kind));
		let mut before = Vec::new();
		let mut after = Vec::new();
		for block in spaced {
			if block.first > 1 && !document.is_blank(block.first - 1) {
				before.push(block.first);
			}
			if !document.is_blank(block.last + 1) {
				after.push(block.last);
			}
		}
		let mut fix = Fix::default();
		for (index, line) in document.lines.iter().enumerate() {
			let number = index + 1;
			if before.contains(&number) && !after.contains(&(number - 1)) {
				fix.lines.push(String::new());
			}
			fix.lines.push(line.text.clone());
			if after.contains(&number) {
				fix.lines.push(String::new());
			}
		}
		fix.changed = before.into_iter().chain(after).collect();
		fix.changed.sort_unstable();
		fix.changed.dedup();
		fix
	}
}
