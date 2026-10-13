//! The trailing-whitespace formatter: no spaces or tabs at the end of a line.

use super::{About, Fix, Formatter, Settings, each_prose_line};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct TrailingWhitespace;

impl Formatter for TrailingWhitespace {
	fn about(&self) -> About {
		About {
			name: "trailing-whitespace",
			description: "removes spaces and tabs at the end of a line",
			message: "trailing whitespace",
			since: FIRST_RELEASE,
			example: "A line that ends in spaces.   \nThe next line.\n",
		}
	}

	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		each_prose_line(document, |line| line.text.trim_end_matches([' ', '\t']).to_owned())
	}
}
