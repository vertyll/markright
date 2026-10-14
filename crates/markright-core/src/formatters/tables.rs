//! The tables formatter: every column of a table padded to one width.

use super::width::display_width;
use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::{Document, is_table_row, table_cells};

pub(super) struct Tables;

impl Formatter for Tables {
	fn about(&self) -> About {
		About {
			name: "tables",
			description: "pads every column of a table to one width",
			message: "table columns are not aligned",
			since: FIRST_RELEASE,
			example: "| Name | Role |\n|---|---|\n| markright | formats and lints Markdown |\n",
		}
	}

	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		by_line(&document.lines, align(document))
	}
}

/// Pads the tables the parser found, from the header through the rows that follow it written with a leading `|`. A
/// row the parser counts without one, such as a line of text right after the table, which GFM makes a row, stays as
/// written: padding would not change how it renders. A table whose header or delimiter row is not written that way,
/// as in a blockquote, is left alone, and so is one with a row wider than its header, which table-columns reports.
fn align(document: &Document) -> Vec<String> {
	let mut out = texts(document);
	for table in &document.tables {
		// The delimiter row, right under the header, is part of the table too, and is its last line when there are
		// no body rows.
		let first = table.header - 1;
		let last = table.body.last().copied().unwrap_or(0).max(first + 2);
		let end = (first..last)
			.find(|&i| document.lines[i].code || !is_table_row(&document.lines[i].text))
			.unwrap_or(last);
		let block = &out[first..end];
		if block.len() < 2 || has_extra_cells(block) {
			continue;
		}
		let indent = block[0][..block[0].len() - block[0].trim_start_matches([' ', '\t']).len()].to_owned();
		for (offset, row) in format(block).into_iter().enumerate() {
			out[first + offset] = format!("{indent}{row}");
		}
	}
	out
}

/// The rows of `block`, a table whose second row is its delimiter row, with every column padded to one width.
fn format(block: &[String]) -> Vec<String> {
	let mut rows: Vec<Vec<String>> = block.iter().map(|line| table_cells(line)).collect();
	let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
	let mut widths = vec![0; columns];
	for (index, row) in rows.iter_mut().enumerate() {
		row.resize(columns, String::new());
		if index != 1 {
			for (width, cell) in widths.iter_mut().zip(row.iter()) {
				*width = (*width).max(display_width(cell));
			}
		}
	}
	let delimiter = rows[1].clone();
	rows.iter()
		.enumerate()
		.map(|(index, row)| {
			let cells = row.iter().zip(&widths).zip(&delimiter).map(|((cell, &width), align)| {
				if index == 1 {
					delimiter_cell(cell, width + 2)
				} else {
					format!(" {} ", pad(cell, width, align))
				}
			});
			format!("|{}|", cells.collect::<Vec<_>>().join("|"))
		})
		.collect()
}

/// Pads `cell` to `width` columns, on the side the column's delimiter cell aligns it to.
fn pad(cell: &str, width: usize, delimiter: &str) -> String {
	let gap = width - display_width(cell);
	let left = match (delimiter.starts_with(':'), delimiter.ends_with(':')) {
		(true, true) => gap / 2,
		(false, true) => gap,
		_ => 0,
	};
	format!("{}{cell}{}", " ".repeat(left), " ".repeat(gap - left))
}

/// A delimiter cell `width` dashes long, keeping the colons that align its column.
fn delimiter_cell(cell: &str, width: usize) -> String {
	let mut dashes = vec![b'-'; width];
	if cell.starts_with(':') {
		dashes[0] = b':';
	}
	if cell.ends_with(':') {
		dashes[width - 1] = b':';
	}
	String::from_utf8(dashes).expect("dashes and colons are ASCII")
}

/// Whether a row below the header has more cells than the header.
fn has_extra_cells(rows: &[String]) -> bool {
	let header = table_cells(&rows[0]).len();
	rows[1..].iter().any(|row| table_cells(row).len() > header)
}
