use super::{About, Fix, Formatter, Settings, V0_1_0};
use crate::markdown::Document;
use crate::markdown::Line;

pub(super) struct BlankLines;

impl Formatter for BlankLines {
	fn about(&self) -> About {
		About {
			name: "blank-lines",
			description: "leaves at most one blank line in a row outside code",
			message: "more than one blank line in a row",
			since: V0_1_0,
			example: "First paragraph.\n\n\n\nSecond paragraph.\n",
		}
	}

	fn apply(&self, d: &Document, _: &Settings) -> Fix {
		let blank = |line: &Line| !line.code && line.text.trim().is_empty();
		let end = content_end(&d.lines);
		let mut fix = Fix {
			lines: Vec::with_capacity(d.lines.len()),
			changed: Vec::new(),
		};
		for (i, line) in d.lines.iter().enumerate() {
			if i > 0 && i < end && blank(line) && blank(&d.lines[i - 1]) {
				fix.changed.push(i + 1);
			} else {
				fix.lines.push(line.text.clone());
			}
		}
		fix
	}
}

/// How many lines there are up to the last one that is not blank.
pub(super) fn content_end(lines: &[Line]) -> usize {
	lines
		.iter()
		.rposition(|line| !line.text.trim().is_empty())
		.map_or(0, |i| i + 1)
}
