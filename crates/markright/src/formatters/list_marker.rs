use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct ListMarker;

impl Formatter for ListMarker {
	fn about(&self) -> About {
		About {
			name: "list-marker",
			description: "marks the items of an unordered list with `-`, or the marker its settings choose",
			message: "list item marked differently from the configured marker",
			since: FIRST_RELEASE,
			example: "* first\n* second\n",
		}
	}

	fn apply(&self, d: &Document, settings: &Settings) -> Fix {
		let marker = settings.list_marker.marker;
		let mut out = texts(d);
		for bullet in &d.bullets {
			let i = bullet.line - 1;
			if bullet.marker != marker && !bullet.adjacent && !d.lines[i].code {
				out[i].replace_range(
					bullet.column..bullet.column + bullet.marker.as_str().len(),
					marker.as_str(),
				);
			}
		}
		by_line(&d.lines, out)
	}
}
