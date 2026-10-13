//! Where a link destination is written in the source: the parser gives the destination itself, resolved, and these
//! find the bytes it was written in, so formatters can leave them alone.

use std::ops::Range;

/// The bytes of the destination of the inline link whose text ends at `text_end`, inside any angle brackets.
pub(super) fn inline(source: &str, text_end: usize) -> Option<Range<usize>> {
	let close = text_end + source[text_end..].find(']')?;
	if source.as_bytes().get(close + 1) != Some(&b'(') {
		return None;
	}
	destination(source, close + 2)
}

/// The bytes of the destination of the reference definition at `span`, inside any angle brackets.
pub(super) fn definition(source: &str, span: &Range<usize>) -> Option<Range<usize>> {
	let bytes = source.as_bytes();
	let mut k = span.start + source[span.clone()].find('[')? + 1;
	while k < span.end && bytes[k] != b']' {
		k += if bytes[k] == b'\\' { 2 } else { 1 };
	}
	if bytes.get(k + 1) != Some(&b':') {
		return None;
	}
	destination(source, k + 2)
}

/// The destination starting at `k`, after any space and up to one line break: a run in angle brackets, or a run up to
/// the first space or unbalanced `)`. None when it is empty.
fn destination(source: &str, k: usize) -> Option<Range<usize>> {
	let bytes = source.as_bytes();
	let start = k + source[k..].find(|c: char| !matches!(c, ' ' | '\t' | '\n' | '\r'))?;
	let span = if bytes[start] == b'<' {
		let end = start + 1 + source[start + 1..].find(['<', '>', '\n'])?;
		(bytes[end] == b'>').then_some(start + 1..end)?
	} else {
		start..bare_end(bytes, start)
	};
	(!span.is_empty()).then_some(span)
}

fn bare_end(bytes: &[u8], mut k: usize) -> usize {
	let mut depth = 0usize;
	while k < bytes.len() {
		match bytes[k] {
			b'\\' if k + 1 < bytes.len() => k += 1,
			b'(' => depth += 1,
			b')' if depth == 0 => return k,
			b')' => depth -= 1,
			b' ' | b'\t' | b'\n' | b'\r' => return k,
			_ => {}
		}
		k += 1;
	}
	k
}
