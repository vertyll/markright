//! The table-pipes formatter: every row of a table starts and ends with a pipe.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

pub(super) struct TablePipes;

impl Formatter for TablePipes {
	fn about(&self) -> About {
		About {
			name: "table-pipes",
			description: "starts and ends every row of a table with a pipe",
			message: "table row without a leading or trailing pipe",
			since: FIRST_RELEASE,
			example: "Name | Role\n--- | ---\nmarkright | formats Markdown\n",
		}
	}

	/// Leaves a table in a blockquote alone, where a `|` would have to go after the `>`.
	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		let mut out = texts(document);
		for table in &document.tables {
			if document.lines[table.header - 1].text.trim_start().starts_with('>') {
				continue;
			}
			let rows = [table.header, table.header + 1]
				.into_iter()
				.chain(table.body.iter().copied());
			for index in rows.map(|line| line - 1) {
				if let Some(enclosed) = enclose(&out[index]).filter(|_| !document.lines[index].code) {
					out[index] = enclosed;
				}
			}
		}
		by_line(&document.lines, out)
	}
}

/// `row` with the `|` it lacks at its start or end, and its indentation kept, or nothing when it has both.
fn enclose(row: &str) -> Option<String> {
	let cells = row.trim();
	if cells.is_empty() {
		return None;
	}
	let indent = &row[..row.len() - row.trim_start().len()];
	let leading = if cells.starts_with('|') { "" } else { "| " };
	let trailing = if ends_with_pipe(cells) { "" } else { " |" };
	(leading.len() + trailing.len() > 0).then(|| format!("{indent}{leading}{cells}{trailing}"))
}

/// Whether `cells` ends with a `|` that is not escaped, which would be text in the last cell.
fn ends_with_pipe(cells: &str) -> bool {
	cells.strip_suffix('|').is_some_and(|before| {
		let backslashes = before.len() - before.trim_end_matches('\\').len();
		backslashes % 2 == 0
	})
}
