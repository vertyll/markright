//! The tables of `docs/rules.md` and `docs/configuration.md`, written between their markers from the registries and
//! the JSON schema, so they never drift from the code.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fs;
use std::io;
use std::path::Path;

use markright::markdown::Document;
use markright::{formatters, linters};
use serde_json::Value;

/// A generated table: the document it is in, the name of its markers, its header and its rows.
struct Table {
	document: &'static str,
	marker: &'static str,
	header: &'static [&'static str],
	rows: Vec<Vec<String>>,
}

fn tables(root: &Path) -> io::Result<Vec<Table>> {
	let code = |s: &str| format!("`{s}`");
	Ok(vec![
		Table {
			document: "docs/rules.md",
			marker: "formatters",
			header: &["Formatter", "What it does"],
			rows: formatters::ALL
				.iter()
				.map(|f| f.about())
				.map(|a| vec![code(a.name), a.description.to_owned()])
				.collect(),
		},
		Table {
			document: "docs/rules.md",
			marker: "linters",
			header: &["Linter", "What it enforces"],
			rows: linters::ALL
				.iter()
				.map(|l| l.about())
				.map(|a| vec![code(a.name), a.description.to_owned()])
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

/// One row per setting: its key, what it means and its default, read from the schema, the formatters first, each
/// section in the order of its registry.
fn settings(root: &Path) -> io::Result<Vec<Vec<String>>> {
	let raw = fs::read_to_string(root.join(crate::schema::PATH))?;
	let schema: Value = serde_json::from_str(&raw).map_err(io::Error::other)?;
	let resolve = |node: &'_ Value| -> Value {
		let name = node["$ref"].as_str().and_then(|r| r.strip_prefix("#/$defs/"));
		name.map_or_else(|| node.clone(), |name| schema["$defs"][name].clone())
	};
	let sections = [
		(
			"formatters",
			formatters::ALL.iter().map(|f| f.about().name).collect::<Vec<_>>(),
		),
		("linters", linters::ALL.iter().map(|l| l.about().name).collect()),
	];
	let mut rows = Vec::new();
	for (section, names) in sections {
		let rules = resolve(&resolve(&schema["properties"][section])["properties"]["settings"]);
		for name in names {
			let rule = resolve(&rules["properties"][name]);
			let Some(keys) = rule["properties"].as_object() else {
				continue;
			};
			for (key, property) in keys {
				let meaning = property["description"].as_str().unwrap_or_default();
				let meaning = lowercase_first(meaning.trim_end_matches('.'));
				let default = default(&property["default"]);
				rows.push(vec![format!("`{section}.settings.{name}.{key}`"), meaning, default]);
			}
		}
	}
	Ok(rows)
}

fn lowercase_first(text: &str) -> String {
	let mut chars = text.chars();
	chars
		.next()
		.map(|c| c.to_lowercase().chain(chars).collect())
		.unwrap_or_default()
}

fn default(value: &Value) -> String {
	match value {
		Value::Array(items) => items.iter().map(default).collect::<Vec<_>>().join(", "),
		Value::String(s) => format!("`{s}`"),
		other => format!("`{other}`"),
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
	#[test]
	fn the_tables_are_current() {
		super::check(&crate::root()).unwrap();
	}
}
