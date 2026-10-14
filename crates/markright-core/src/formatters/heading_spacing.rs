//! The heading-spacing formatter: a heading written with `#` takes one space after the `#` and no closing `#`.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::{BlockKind, Document};

pub(super) struct HeadingSpacing;

impl Formatter for HeadingSpacing {
	fn about(&self) -> About {
		About {
			name: "heading-spacing",
			description: "writes a heading as its `#`, one space and its text, without indentation or closing `#`",
			message: "heading not written as its `#`, one space and its text",
			since: FIRST_RELEASE,
			example: "##   Install ##\n",
		}
	}

	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		let mut out = texts(document);
		for heading in &document.headings {
			let top_level = document.top_level(BlockKind::Heading, heading.line).is_some();
			if heading.underline.is_none() && top_level {
				let index = heading.line - 1;
				out[index] = normalize(&out[index], heading.level);
			}
		}
		by_line(&document.lines, out)
	}
}

/// `line`, a heading of `level` written with `#`, as its `#`, one space and its text.
fn normalize(line: &str, level: u8) -> String {
	let text = line.trim_start().trim_start_matches('#').trim();
	let without_closing = text.trim_end_matches('#');
	// A closing run of `#` follows a space or stands alone; one right after the text, as in `C#`, is the text.
	let text = if without_closing.is_empty() || without_closing.ends_with([' ', '\t']) {
		without_closing.trim_end()
	} else {
		text
	};
	let marker = "#".repeat(usize::from(level));
	if text.is_empty() {
		marker
	} else {
		format!("{marker} {text}")
	}
}
