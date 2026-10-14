//! The list-marker-space formatter: one space between a list marker and the text of a one-line item.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

/// The most spaces after a marker that still start the item's text; five or more start an indented code block.
const MOST_SPACES: usize = 4;

pub(super) struct ListMarkerSpace;

impl Formatter for ListMarkerSpace {
	fn about(&self) -> About {
		About {
			name: "list-marker-space",
			description: "leaves one space between the marker and the text of a list item",
			message: "more than one space after a list marker",
			since: FIRST_RELEASE,
			example: "-   first\n-   second\n",
		}
	}

	/// Only items of one line change: moving the text of a longer one would move the column its other lines are
	/// measured from.
	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		let mut out = texts(document);
		for item in &document.items {
			let index = item.line - 1;
			let after = item.column + item.marker.width();
			let one_line = item.last_line == item.line && !document.lines[index].code;
			let text_follows = out[index].len() > after + item.spaces;
			if one_line && text_follows && (2..=MOST_SPACES).contains(&item.spaces) {
				out[index].replace_range(after..after + item.spaces, " ");
			}
		}
		by_line(&document.lines, out)
	}
}
