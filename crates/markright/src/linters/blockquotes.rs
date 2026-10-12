use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

pub(super) struct Blockquotes;

impl DocumentLinter for Blockquotes {
	fn about(&self) -> About {
		About {
			name: "blockquotes",
			description: "a line that continues a blockquote's paragraph starts with `>`",
			since: FIRST_RELEASE,
			example: Example::new(
				"> A quote that\ngoes on without the marker.\n",
				"> A quote that\n> goes on with the marker.\n",
			),
		}
	}

	fn check(&self, _: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		Ok(doc
			.lazy_lines
			.iter()
			.map(|&line| Finding::new(path, line, "blockquote continues on a line without >"))
			.collect())
	}
}
