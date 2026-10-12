//! The anchors a document defines: GitHub's slug of each heading, and the ids of HTML elements.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

static HTML_ID: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r#"<[A-Za-z][^>]*?[\t\n\f\r ](?:id|name)[\t\n\f\r ]*=[\t\n\f\r ]*["']([^"']+)["']"#)
		.expect("the HTML id pattern is valid")
});

/// Collects anchors into a set, numbering a heading's slug when an earlier heading had it.
#[derive(Default)]
pub(super) struct Anchors {
	seen: HashMap<String, usize>,
	set: HashSet<String>,
}

impl Anchors {
	/// The anchors collected.
	pub(super) fn into_set(self) -> HashSet<String> {
		self.set
	}

	/// Adds the anchor of a heading whose text, without markup, is `text`.
	pub(super) fn heading(&mut self, text: &str) {
		let slug = slugify(text);
		let count = *self
			.seen
			.entry(slug.clone())
			.and_modify(|count| *count += 1)
			.or_insert(0);
		let anchor = if count == 0 { slug } else { format!("{slug}-{count}") };
		self.set.insert(anchor);
	}

	/// Adds the id or name of every HTML element in `html`.
	pub(super) fn html(&mut self, html: &str) {
		for captures in HTML_ID.captures_iter(html) {
			self.set.insert(captures[1].to_lowercase());
		}
	}
}

/// The anchor GitHub derives from a heading's text: lowercased, with letters, digits, `-` and `_` kept, spaces turned
/// into `-`, and everything else dropped. The suffix of a repeated heading is not added.
pub(super) fn slugify(text: &str) -> String {
	text.chars()
		.flat_map(char::to_lowercase)
		.filter_map(|c| match c {
			' ' => Some('-'),
			'-' | '_' => Some(c),
			_ if matches!(
				c.general_category_group(),
				GeneralCategoryGroup::Letter | GeneralCategoryGroup::Number
			) =>
			{
				Some(c)
			}
			_ => None,
		})
		.collect()
}
