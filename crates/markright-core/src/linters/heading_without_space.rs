//! The heading-without-space linter: a line that starts with `#` but no space after it, which is text, not a heading.

use std::sync::LazyLock;

use regex::Regex;

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

/// One to six `#` after up to three spaces, then something other than a space or another `#`.
static UNSPACED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^ {0,3}#{1,6}[^#\s]").expect("valid pattern"));

pub(super) struct HeadingWithoutSpace;

impl DocumentLinter for HeadingWithoutSpace {
	fn about(&self) -> About {
		About {
			name: "heading-without-space",
			description: "a line meant as a heading has a space after its `#`",
			since: FIRST_RELEASE,
			example: Example::new("#Install\n", "# Install\n"),
		}
	}

	fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
		let headings: Vec<usize> = document.headings.iter().map(|heading| heading.line).collect();
		Ok(document
			.lines
			.iter()
			.enumerate()
			.filter(|(index, line)| !line.code && !headings.contains(&(index + 1)) && UNSPACED.is_match(&line.text))
			.map(|(index, _)| {
				Finding::new(
					path,
					index + 1,
					"`#` without a space after it, so the line is no heading",
				)
			})
			.collect())
	}
}
