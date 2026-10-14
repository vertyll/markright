//! The fence-marker formatter: the fences of code blocks written with one character, a backtick or a tilde.

use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::{Document, Fence};

/// The fewest characters a fence takes.
const SHORTEST: usize = 3;

pub(super) struct FenceMarker;

impl Formatter for FenceMarker {
	fn about(&self) -> About {
		About {
			name: "fence-marker",
			description: "writes the fences of code blocks with backticks, or the character its settings choose",
			message: "code fence written with a different character from the configured one",
			since: FIRST_RELEASE,
			example: "~~~bash\nmake build\n~~~\n",
		}
	}

	/// The new fence is longer than any run of its character that starts a line of the code, so none of them closes
	/// it. A block that is never closed stays as written, and so does one whose info string holds a backtick, which a
	/// backtick fence does not allow.
	fn apply(&self, document: &Document, settings: &Settings) -> Fix {
		let marker = settings.fence_marker.marker.as_char();
		let mut out = texts(document);
		for fence in &document.fences {
			let Some(closing) = fence.closing else { continue };
			if fence.marker == marker || (marker == '`' && fence.info.contains('`')) {
				continue;
			}
			let longest = (fence.line..closing - 1)
				.map(|index| run(out[index].trim_start_matches([' ', '>']), marker))
				.max()
				.unwrap_or(0);
			let new = marker.to_string().repeat(fence.length.max(longest + 1).max(SHORTEST));
			for line in [fence.line, closing] {
				replace_run(&mut out[line - 1], fence, &new);
			}
		}
		by_line(&document.lines, out)
	}
}

/// How many of `marker` `text` starts with.
fn run(text: &str, marker: char) -> usize {
	text.chars().take_while(|&c| c == marker).count()
}

/// Replaces the run of the fence's character in `line`, a fence line of it, with `new`.
fn replace_run(line: &mut String, fence: &Fence, new: &str) {
	let Some(start) = line.find(fence.marker) else { return };
	let length = run(&line[start..], fence.marker);
	line.replace_range(start..start + length, new);
}
