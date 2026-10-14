//! The reversed-links linter: a link written `(text)[target]`, with its brackets swapped, which is no link.

use std::sync::LazyLock;

use regex::Regex;

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{Document, normalize_label};
use crate::{Error, FIRST_RELEASE};

/// Text in parentheses right before text in brackets, with no parenthesis after it.
static REVERSED: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"\(([^()\n]+)\)\[([^\[\]\n]+)\]").expect("valid pattern"));

pub(super) struct ReversedLinks;

impl DocumentLinter for ReversedLinks {
	fn about(&self) -> About {
		About {
			name: "reversed-links",
			description: "a link is written `[text](target)`, not with its brackets swapped",
			since: FIRST_RELEASE,
			example: Example::new("See (the guide)[guide.md].\n", "See [the guide](guide.md).\n"),
		}
	}

	/// What follows a link or a reference, `[a](b)[c]` or `(see)[label]` with `label` defined, is no reversed link;
	/// nor is anything in code or a protected span.
	fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		let mut found = Vec::new();
		for (index, line) in document.lines.iter().enumerate().filter(|(_, line)| !line.code) {
			for captures in REVERSED.captures_iter(&line.text) {
				let whole = captures.get(0).expect("a match has its whole text");
				let protected = line
					.protected
					.iter()
					.any(|range| range.start < whole.end() && whole.start() < range.end);
				let after_link = line.text[..whole.start()].ends_with([']', '\\']);
				let followed = line.text[whole.end()..].starts_with('(');
				let reference = document.definitions.contains_key(&normalize_label(&captures[2]));
				if !(protected || after_link || followed || reference) {
					found.push(Finding::new(
						path,
						index + 1,
						format!("link with its brackets swapped: {}", whole.as_str()),
					));
				}
			}
		}
		Ok(found)
	}
}
