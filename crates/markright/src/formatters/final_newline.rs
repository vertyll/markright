use super::blank_lines::content_end;
use super::{About, Fix, Formatter, Settings, texts};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct FinalNewline;

impl Formatter for FinalNewline {
	fn about(&self) -> About {
		About {
			name: "final-newline",
			description: "ends the file with exactly one newline",
			message: "the file does not end with exactly one newline",
			since: FIRST_RELEASE,
			example: "The last line, without a newline",
		}
	}

	fn apply(&self, d: &Document, _: &Settings) -> Fix {
		let lines = &d.lines;
		let end = content_end(lines);
		if end == 0 || (end == lines.len() - 1 && lines[end].text.is_empty()) {
			return Fix {
				lines: texts(d),
				changed: Vec::new(),
			};
		}
		let mut out = texts(d);
		out.truncate(end);
		out.push(String::new());
		let changed = if end == lines.len() { lines.len() } else { end + 1 };
		Fix {
			lines: out,
			changed: vec![changed],
		}
	}
}
