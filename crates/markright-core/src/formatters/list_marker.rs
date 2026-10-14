//! The list-marker formatter: the items of unordered lists take the configured marker.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::{Document, ItemMarker};

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

	fn apply(&self, document: &Document, settings: &Settings) -> Fix {
		let marker = settings.list_marker.marker;
		let mut out = texts(document);
		for item in &document.items {
			let ItemMarker::Bullet(written) = item.marker else {
				continue;
			};
			let index = item.line - 1;
			if written != marker && !item.adjacent && !document.lines[index].code {
				out[index].replace_range(item.column..=item.column, marker.as_str());
			}
		}
		by_line(&document.lines, out)
	}
}
