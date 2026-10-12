//! The cells of a table row, as the formatters and linters count and align them.

/// Whether `line` starts with `|`, as a table row written with a leading pipe does. Which lines form a table is for
/// the parser to say: GFM also takes rows without one.
#[must_use]
pub fn is_table_row(line: &str) -> bool {
	line.trim().starts_with('|')
}

/// Splits a table row into its cells, trimmed, keeping pipes inside inline code and escaped pipes in their cell.
#[must_use]
pub fn table_cells(line: &str) -> Vec<String> {
	let trimmed = line.trim();
	let body = trimmed.strip_prefix('|').unwrap_or(trimmed);
	let body = match body.strip_suffix('|') {
		Some(rest) if !rest.ends_with('\\') => rest,
		_ => body,
	};
	let bytes = body.as_bytes();
	let mut cells = Vec::new();
	let mut start = 0;
	let mut i = 0;
	while i < bytes.len() {
		match bytes[i] {
			b'\\' if i + 1 < bytes.len() => i += 2,
			b'`' => i = code_span_end(bytes, i),
			b'|' => {
				cells.push(body[start..i].trim().to_owned());
				i += 1;
				start = i;
			}
			_ => i += 1,
		}
	}
	cells.push(body[start..].trim().to_owned());
	cells
}

/// Where the code span opening at `start` ends: after the next run of as many backticks, or right after the opening
/// run when none closes it.
fn code_span_end(bytes: &[u8], start: usize) -> usize {
	let n = backtick_run(bytes, start);
	let mut i = start + n;
	while i < bytes.len() {
		if bytes[i] == b'`' {
			let m = backtick_run(bytes, i);
			if m == n {
				return i + m;
			}
			i += m;
		} else {
			i += 1;
		}
	}
	start + n
}

fn backtick_run(bytes: &[u8], i: usize) -> usize {
	bytes[i..].iter().take_while(|&&b| b == b'`').count()
}
