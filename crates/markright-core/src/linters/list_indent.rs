//! The list-indent linter: the items of one list start at one column.

use std::collections::HashMap;

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct ListIndent;

impl DocumentLinter for ListIndent {
	fn about(&self) -> About {
		About {
			name: "list-indent",
			description: "the items of one list are indented alike",
			since: FIRST_RELEASE,
			example: Example::new("# Steps\n\n- one\n - two\n", "# Steps\n\n- one\n- two\n"),
		}
	}

	/// Compares each item with the first of its list: an item is indented alike when its marker or its text starts
	/// at the same column, so numbers aligned on the right, as ` 9.` over `10.`, are fine.
	fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		let mut first = HashMap::new();
		let mut out = Vec::new();
		for item in &document.items {
			let text = item.column + item.marker.width() + item.spaces;
			let &mut (marker, start) = first.entry(item.list).or_insert((item.column, text));
			if item.column != marker && text != start {
				out.push(Finding::new(
					path,
					item.line,
					"list item indented differently from the first item of its list",
				));
			}
		}
		Ok(out)
	}
}
