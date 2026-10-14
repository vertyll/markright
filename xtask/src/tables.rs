//! The tables of `docs/rules.md` and `docs/configuration.md`, written between their markers from the registries and
//! the JSON schema, so they never drift from the code.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fs;
use std::io;
use std::path::Path;

use markright_core::markdown::Document;
use markright_core::{formatters, linters};
use serde_json::Value;

/// A generated table: the document it is in, the name of its markers, its header and its rows.
struct Table {
	document: &'static str,
	marker: &'static str,
	header: &'static [&'static str],
	rows: Vec<Vec<String>>,
}

/// Every generated table, with its rows as the registries and the schema make them now.
fn tables(root: &Path) -> io::Result<Vec<Table>> {
	let code = |s: &str| format!("`{s}`");
	Ok(vec![
		Table {
			document: "docs/rules.md",
			marker: "formatters",
			header: &["Formatter", "What it does"],
			rows: formatters::ALL
				.iter()
				.map(|formatter| formatter.about())
				.map(|about| vec![code(about.name), about.description.to_owned()])
				.collect(),
		},
		Table {
			document: "docs/rules.md",
			marker: "linters",
			header: &["Linter", "What it enforces"],
			rows: linters::ALL
				.iter()
				.map(|linter| linter.about())
				.map(|about| vec![code(about.name), about.description.to_owned()])
				.collect(),
		},
		Table {
			document: "docs/configuration.md",
			marker: "settings",
			header: &["Key", "Meaning", "Default"],
			rows: settings(root)?,
		},
	])
}

/// One row per setting: its key, what it means and its default, read from the schema.
fn settings(root: &Path) -> io::Result<Vec<Vec<String>>> {
	Ok(crate::schema::settings(root)?
		.into_iter()
		.map(|setting| {
			vec![
				format!("`{}.settings.{}.{}`", setting.section, setting.rule, setting.key),
				lowercase_first(setting.description.trim_end_matches('.')),
				default(&setting.default),
			]
		})
		.collect())
}

/// `text` with its first letter in lower case, as the cells of the tables start.
fn lowercase_first(text: &str) -> String {
	let mut chars = text.chars();
	chars
		.next()
		.map(|first| first.to_lowercase().chain(chars).collect())
		.unwrap_or_default()
}

/// A setting's default as a table cell shows it: each value in a code span.
pub(crate) fn default(value: &Value) -> String {
	match value {
		Value::Array(items) => items.iter().map(default).collect::<Vec<_>>().join(", "),
		Value::String(text) => code_span(text),
		other => code_span(&other.to_string()),
	}
}

/// `text` as a code span: between more backticks than any run of them inside it, with a space on each side when it
/// starts or ends with one, so `` ` `` reads as a backtick.
fn code_span(text: &str) -> String {
	let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
	let fence = "`".repeat(longest + 1);
	if text.starts_with('`') || text.ends_with('`') {
		format!("{fence} {text} {fence}")
	} else {
		format!("{fence}{text}{fence}")
	}
}

impl Table {
	/// The table as Markdown, aligned by the tables formatter of markright itself.
	fn markdown(&self) -> String {
		let header: Vec<String> = self.header.iter().map(|&cell| cell.to_owned()).collect();
		let delimiter = vec!["---".to_owned(); header.len()];
		let lines: Vec<String> = [header, delimiter]
			.iter()
			.chain(&self.rows)
			.map(|cells| row(cells))
			.collect();
		let tables = formatters::find("tables").expect("the tables formatter exists");
		tables
			.apply(
				&Document::parse(&lines.join(
					"
",
				)),
				&formatters::Settings::default(),
			)
			.lines
			.join(
				"
",
			)
	}

	/// `content` with this table between its markers.
	fn place(&self, content: &str) -> io::Result<String> {
		let open = format!("<!-- markright:{} -->\n", self.marker);
		let close = format!("<!-- /markright:{} -->", self.marker);
		let missing = || io::Error::other(format!("{} has no {open:?} and {close:?} markers", self.document));
		let start = content.find(&open).ok_or_else(missing)? + open.len();
		let end = start + content[start..].find(&close).ok_or_else(missing)?;
		Ok(format!(
			"{}\n{}\n\n{}",
			&content[..start],
			self.markdown(),
			&content[end..]
		))
	}
}

/// The cells as one row of a Markdown table.
fn row(cells: &[String]) -> String {
	format!("| {} |", cells.join(" | "))
}

/// Each document with its tables in place, by path, and whether it changed.
fn updated(root: &Path) -> io::Result<BTreeMap<&'static str, (String, bool)>> {
	let mut documents: BTreeMap<&'static str, (String, bool)> = BTreeMap::new();
	for table in tables(root)? {
		let (content, changed) = match documents.entry(table.document) {
			Entry::Occupied(entry) => entry.into_mut(),
			Entry::Vacant(entry) => entry.insert((fs::read_to_string(root.join(table.document))?, false)),
		};
		let placed = table.place(content)?;
		*changed |= placed != *content;
		*content = placed;
	}
	Ok(documents)
}

/// Writes every table between its markers.
pub(crate) fn write(root: &Path) -> io::Result<()> {
	for (path, (content, changed)) in updated(root)? {
		if changed {
			fs::write(root.join(path), content)?;
			println!("wrote {path}");
		}
	}
	Ok(())
}

/// Fails when a table is not what the registries and the schema make of it.
pub(crate) fn check(root: &Path) -> io::Result<()> {
	let stale: Vec<&str> = updated(root)?
		.into_iter()
		.filter(|(_, (_, changed))| *changed)
		.map(|(path, _)| path)
		.collect();
	if stale.is_empty() {
		Ok(())
	} else {
		Err(io::Error::other(format!(
			"out of date: {}; run cargo xtask tables",
			stale.join(", ")
		)))
	}
}

#[cfg(test)]
mod tests {
	use markright_core::markdown::Document;

	use super::{code_span, tables};

	#[test]
	fn the_tables_are_current() {
		super::check(&crate::root()).unwrap();
	}

	#[test]
	fn every_table_has_one_row_per_entry() {
		for table in tables(&crate::root()).unwrap() {
			let parsed = Document::parse(&table.markdown());
			let rows = parsed.tables.first().map_or(0, |parsed| parsed.body.len());
			assert_eq!(rows, table.rows.len(), "{}: {}", table.document, table.marker);
		}
	}

	#[test]
	fn a_code_span_holds_any_backticks() {
		assert_eq!(code_span("-"), "`-`");
		assert_eq!(code_span("`"), "`` ` ``");
		assert_eq!(code_span("a``b"), "```a``b```");
	}
}
