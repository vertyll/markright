//! The hard-tabs formatter: tabs in prose become spaces, up to the next multiple of four columns.

use super::{About, Fix, Formatter, Settings, each_prose_line};
use crate::FIRST_RELEASE;
use crate::markdown::{Document, Line};

/// The columns a tab reaches to, as `CommonMark` counts them.
const TAB_STOP: usize = 4;

pub(super) struct HardTabs;

impl Formatter for HardTabs {
	fn about(&self) -> About {
		About {
			name: "hard-tabs",
			description: "turns tabs outside code into spaces, up to the next multiple of four columns",
			message: "hard tab",
			since: FIRST_RELEASE,
			example: "Name:\tmarkright\n",
		}
	}

	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		each_prose_line(document, expand)
	}
}

/// The line with every tab outside its protected ranges turned into spaces.
fn expand(line: &Line) -> String {
	let mut out = String::with_capacity(line.text.len());
	let mut column = 0;
	for (offset, c) in line.text.char_indices() {
		let protected = line.protected.iter().any(|range| range.contains(&offset));
		if c == '\t' && !protected {
			let width = TAB_STOP - column % TAB_STOP;
			out.extend(std::iter::repeat_n(' ', width));
			column += width;
		} else {
			out.push(c);
			column += 1;
		}
	}
	out
}
