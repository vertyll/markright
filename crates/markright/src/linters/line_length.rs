use std::sync::LazyLock;

use regex::Regex;

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{Document, is_table_row};
use crate::{Error, FIRST_RELEASE};

static LONG_URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://[^\t\n\f\r ]{60,}").expect("valid pattern"));

pub(super) struct LineLength;

impl DocumentLinter for LineLength {
	fn about(&self) -> About {
		About {
			name: "line-length",
			description: "a line is at most the configured number of characters",
			since: FIRST_RELEASE,
			example: Example::new(
				"This sentence keeps going well past the limit of one hundred and twenty characters, so it is hard to read side by side in a diff.\n",
				"This sentence stops before the limit of one hundred and twenty characters\nand goes on on the next line.\n",
			),
		}
	}

	fn check(&self, c: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		let limit = c.settings.line_length.max.get();
		let mut out = Vec::new();
		for (i, line) in doc.lines.iter().enumerate() {
			let n = line.text.chars().count();
			if !line.code && n > limit && !unwrappable(&line.text) {
				out.push(Finding::new(
					path,
					i + 1,
					format!("line is {n} characters, limit {limit}"),
				));
			}
		}
		Ok(out)
	}
}

/// Whether a line cannot be wrapped: a table row, a long URL, HTML or an image.
fn unwrappable(line: &str) -> bool {
	let t = line.trim();
	is_table_row(line) || LONG_URL.is_match(line) || t.starts_with('<') || t.starts_with("[![") || t.starts_with("![")
}
