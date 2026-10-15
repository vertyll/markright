//! The code-block-style linter: code blocks are fenced, not indented.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{BlockKind, Document};
use crate::{Error, FIRST_RELEASE};

pub(super) struct CodeBlockStyle;

impl DocumentLinter for CodeBlockStyle {
    fn about(&self) -> About {
        About {
            name: "code-block-style",
            description: "a code block is fenced, so it can name its language, not indented",
            since: FIRST_RELEASE,
            example: Example::new("Run:\n\n    make build\n", "Run:\n\n```bash\nmake build\n```\n"),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        Ok(document
            .blocks
            .iter()
            .filter(|block| block.kind == BlockKind::IndentedCode)
            .map(|block| Finding::new(path, block.first, "indented code block; fence it instead"))
            .collect())
    }
}
