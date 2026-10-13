//! The pieces of the generated pages: their TOML front matter and their code blocks.

use std::fmt::Write;

/// `text` as a TOML string. JSON writes a string the way TOML reads a basic string.
pub(super) fn string(text: &str) -> String {
	serde_json::to_string(text).expect("a string always serializes")
}

/// The front matter of a page: `fields`, then `extra`, the values already written as TOML.
pub(super) fn front_matter(fields: &[(&str, String)], extra: &[(&str, String)]) -> String {
	let mut text = String::from("+++\n");
	for (key, value) in fields {
		let _ = writeln!(text, "{key} = {value}");
	}
	if !extra.is_empty() {
		text.push_str("\n[extra]\n");
		for (key, value) in extra {
			let _ = writeln!(text, "{key} = {value}");
		}
	}
	text.push_str("+++\n\n");
	text
}

/// `content` as a fenced code block in `language`, with a fence longer than any run of backticks inside it.
pub(super) fn fence(language: &str, content: &str) -> String {
	let mut marker = String::from("```");
	while content.contains(&marker) {
		marker.push('`');
	}
	format!("{marker}{language}\n{}\n{marker}\n\n", content.trim_end_matches('\n'))
}
