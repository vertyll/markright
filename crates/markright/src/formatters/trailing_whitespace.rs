use super::{About, Fix, Formatter, Settings, V0_1_0, each_prose_line};
use crate::markdown::Document;

pub(super) struct TrailingWhitespace;

impl Formatter for TrailingWhitespace {
	fn about(&self) -> About {
		About {
			name: "trailing-whitespace",
			description: "removes spaces and tabs at the end of a line",
			message: "trailing whitespace",
			since: V0_1_0,
			example: "A line that ends in spaces.   \nThe next line.\n",
		}
	}

	fn apply(&self, d: &Document, _: &Settings) -> Fix {
		each_prose_line(d, |line| line.text.trim_end_matches([' ', '\t']).to_owned())
	}
}
