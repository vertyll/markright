//! The link-destination linter: a link's destination has no space, which leaves `[text](my file.md)` as text, no link.

use std::sync::LazyLock;

use regex::Regex;

use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example};
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

/// Text in brackets, then parentheses holding a destination with a space in it that a title does not explain: what
/// follows the space starts with no `"`, `'` or `(`.
static SPACED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\[[^\[\]\n]*\]\([^()<>\s]+[ \t]+[^()"'\s][^()\n]*\)"#).expect("valid pattern"));

pub(super) struct LinkDestination;

impl DocumentLinter for LinkDestination {
    fn about(&self) -> About {
        About {
            name: "link-destination",
            description: "a link's destination has no space, which leaves `[text](my file.md)` as text: write it `%20`, or wrap the destination in `<>`",
            since: "v0.3.0",
            example: Example::new(
                "See the [guide](docs/user guide.md).\n",
                "See the [guide](docs/guide.md).\n",
            )
            .with_files(&[EXAMPLE_GUIDE]),
        }
    }

    /// A match in code or a protected span, such as the destination of a link the parser did read, is left alone.
    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let mut found = Vec::new();
        for (index, line) in document.lines.iter().enumerate().filter(|(_, line)| !line.code) {
            for whole in SPACED.find_iter(&line.text) {
                let protected = line
                    .protected
                    .iter()
                    .any(|range| range.start < whole.end() && whole.start() < range.end);
                if !protected {
                    found.push(Finding::new(
                        path,
                        index + 1,
                        format!(
                            "link with a space in its destination, which leaves it as text: {}",
                            whole.as_str()
                        ),
                    ));
                }
            }
        }
        Ok(found)
    }
}
