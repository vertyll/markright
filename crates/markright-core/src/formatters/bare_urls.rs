//! The bare-urls formatter: a URL written as plain text becomes a link, in angle brackets.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct BareUrls;

impl Formatter for BareUrls {
	fn about(&self) -> About {
		About {
			name: "bare-urls",
			description: "puts a URL written as plain text in angle brackets, which makes it a link in any renderer",
			message: "URL without angle brackets",
			since: FIRST_RELEASE,
			example: "The releases are at https://github.com/vertyll/markright/releases.\n",
		}
	}

	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		let mut out = texts(document);
		// Right to left within a line, so each edit leaves the bytes of the ones before it where they were.
		for url in document.bare_urls.iter().rev() {
			let index = url.line - 1;
			if !document.lines[index].code {
				out[index].insert(url.columns.end, '>');
				out[index].insert(url.columns.start, '<');
			}
		}
		by_line(&document.lines, out)
	}
}
