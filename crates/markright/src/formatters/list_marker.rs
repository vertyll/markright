use super::{About, Fix, Formatter, Settings, V0_1_0, by_line, texts};
use crate::markdown::Document;

pub(super) struct ListMarker;

impl Formatter for ListMarker {
	fn about(&self) -> About {
		About {
			name: "list-marker",
			description: "marks the items of an unordered list with `-`, or the marker its settings choose",
			message: "list item marked differently from the configured marker",
			since: V0_1_0,
			example: "* first\n* second\n",
		}
	}

	fn apply(&self, d: &Document, settings: &Settings) -> Fix {
		let marker = settings.list_marker.marker;
		let mut out = texts(d);
		for bullet in &d.bullets {
			let i = bullet.line - 1;
			if bullet.marker != marker.as_char() && !bullet.adjacent && !d.lines[i].code {
				out[i].replace_range(bullet.column..bullet.column + bullet.marker.len_utf8(), marker.as_str());
			}
		}
		by_line(&d.lines, out)
	}
}
