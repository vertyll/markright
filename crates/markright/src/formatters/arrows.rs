//! The arrows formatter: ASCII arrows become Unicode ones, with a space on each side.

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use super::{About, Fix, Formatter, Settings, each_prose_line};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

const ARROWS: [char; 3] = ['→', '←', '↔'];

static BOTH: LazyLock<Regex> = LazyLock::new(|| Regex::new("<-+>").expect("valid pattern"));
static RIGHT: LazyLock<Regex> = LazyLock::new(|| Regex::new("-+>").expect("valid pattern"));
static LEFT: LazyLock<Regex> = LazyLock::new(|| Regex::new("<-+").expect("valid pattern"));

pub(super) struct Arrows;

impl Formatter for Arrows {
	fn about(&self) -> About {
		About {
			name: "arrows",
			description: "`->`, `<-` and `<->` become `→`, `←` and `↔`, with a space on each side",
			message: "ASCII arrow or missing space around an arrow",
			since: FIRST_RELEASE,
			example: "Request -> gateway -> service\n",
		}
	}

	fn apply(&self, document: &Document, _: &Settings) -> Fix {
		each_prose_line(document, |line| arrows_in_line(&line.text, &line.protected))
	}
}

/// Converts the arrows of `line` outside its protected ranges, each stretch between them on its own.
fn arrows_in_line(line: &str, protected: &[Range<usize>]) -> String {
	let mut out = String::with_capacity(line.len());
	let mut pos = 0;
	for span in protected {
		if span.start < pos {
			continue;
		}
		out.push_str(&convert(&line[pos..span.start]));
		out.push_str(&line[span.clone()]);
		pos = span.end;
	}
	out.push_str(&convert(&line[pos..]));
	out
}

fn convert(text: &str) -> String {
	let text = BOTH.replace_all(text, "↔");
	let text = RIGHT.replace_all(&text, "→");
	let text = LEFT.replace_all(&text, "←");
	space(&text)
}

/// Puts a space before and after every arrow that has none.
fn space(text: &str) -> String {
	let mut out = String::with_capacity(text.len());
	let mut after_arrow = false;
	for c in text.chars() {
		let arrow = ARROWS.contains(&c);
		let needs_space = out.chars().next_back().is_some_and(|last| !last.is_whitespace());
		if (arrow || after_arrow) && needs_space && !c.is_whitespace() {
			out.push(' ');
		}
		out.push(c);
		after_arrow = arrow;
	}
	out
}
