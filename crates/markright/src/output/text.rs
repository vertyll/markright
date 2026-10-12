use std::io::{self, Write};

use crate::issue::Issue;

/// The issues as the text format writes them: one line each, then a summary.
#[must_use]
pub fn text(issues: &[Issue]) -> String {
	let summary = match issues.len() {
		0 => "markright: 0 issues\n".to_owned(),
		n => format!("markright: {n} issue(s)\n"),
	};
	issues
		.iter()
		.map(|issue| format!("{issue}\n"))
		.chain(std::iter::once(summary))
		.collect()
}

/// Writes the issues as the text format does.
pub(super) fn write(w: &mut dyn Write, issues: &[Issue]) -> io::Result<()> {
	w.write_all(text(issues).as_bytes())
}
