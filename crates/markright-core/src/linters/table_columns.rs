//! The table-columns linter: no table row has more cells than its header.

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::{Document, table_cells};
use crate::{Error, FIRST_RELEASE};

pub(super) struct TableColumns;

impl DocumentLinter for TableColumns {
    fn about(&self) -> About {
        About {
            name: "table-columns",
            description: "a table row has no more cells than the header",
            since: FIRST_RELEASE,
            example: Example::new(
                "| Name | Role |\n|---|---|\n| markright | linter | formatter |\n",
                "| Name | Role |\n|---|---|\n| markright | linter and formatter |\n",
            ),
        }
    }

    fn check(&self, _: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error> {
        let cells = |line: usize| table_cells(&document.lines[line - 1].text).len();
        let mut out = Vec::new();
        for table in &document.tables {
            let header = cells(table.header);
            for &row in &table.body {
                let n = cells(row);
                if n > header {
                    let message = format!("row has {n} cells, the header {header}; the extra cells are not shown");
                    out.push(Finding::new(path, row, message));
                }
            }
        }
        Ok(out)
    }
}
