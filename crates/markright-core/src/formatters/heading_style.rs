//! The heading-style formatter: a heading underlined with `===` or `---` becomes one written with `#`.

use super::{About, Fix, Formatter, Settings};
use crate::FIRST_RELEASE;
use crate::markdown::{BlockKind, Document};

pub(super) struct HeadingStyle;

impl Formatter for HeadingStyle {
	fn about(&self) -> About {
		About {
			name: "heading-style",
			description: "writes a heading underlined with `===` or `---` with `#` instead, on one line",
			message: "heading underlined instead of written with `#`",
			since: FIRST_RELEASE,
			example: "Install\n=======\n\nRun the installer.\n",
		}
	}

	/// The lines of the heading's text become one; the underline goes.
	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		let mut fix = Fix::default();
		let mut skip_to = 0;
		for (index, line) in document.lines.iter().enumerate() {
			if index < skip_to {
				continue;
			}
			let number = index + 1;
			let underlined = document
				.headings
				.iter()
				.find(|heading| heading.line == number && document.top_level(BlockKind::Heading, number).is_some())
				.and_then(|heading| heading.underline.map(|underline| (heading.level, underline)));
			let Some((level, underline)) = underlined else {
				fix.lines.push(line.text.clone());
				continue;
			};
			let text: Vec<&str> = document.lines[index..underline - 1]
				.iter()
				.map(|line| line.text.trim())
				.collect();
			fix.lines
				.push(format!("{} {}", "#".repeat(usize::from(level)), text.join(" ")));
			fix.changed.extend(number..=underline);
			skip_to = underline;
		}
		fix
	}
}
