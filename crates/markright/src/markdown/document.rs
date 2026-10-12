//! The model of a parsed document: its lines, and what the parser found in them.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::ops::Range;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::walker;

/// A parsed Markdown file.
///
/// Line numbers are 1-based, as issues report them; indexes into [`Document::lines`] are 0-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
	/// The lines of the file, without their line endings.
	pub lines: Vec<Line>,
	/// The anchors a `#fragment` may name: one per heading, numbered when repeated, and the ids of HTML elements.
	pub anchors: HashSet<String>,
	/// The headings, in order.
	pub headings: Vec<Heading>,
	/// The fenced code blocks, in order.
	pub fences: Vec<Fence>,
	/// The markers of the items of unordered lists, in order.
	pub bullets: Vec<Bullet>,
	/// The tables, in order.
	pub tables: Vec<Table>,
	/// The inline links and images, in the order they start, then the reference definitions.
	pub links: Vec<Link>,
	/// The reference definitions, by normalized label; the first of a repeated label wins, as it does in the parse.
	pub definitions: BTreeMap<String, Definition>,
	/// The normalized labels that reference links and images resolve through a definition.
	pub used_labels: HashSet<String>,
	/// The full and collapsed reference links, `[text][label]` and `[label][]`, whose label nothing defines.
	pub undefined_references: Vec<Reference>,
	/// The lines that continue a blockquote's paragraph without starting with `>`.
	pub lazy_lines: Vec<usize>,
	/// The line of the first fenced code block that is never closed.
	pub unclosed_fence: Option<usize>,
}

/// One line of a document, and what the formatters must know about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
	/// The line, without its line ending.
	pub text: String,
	/// Whether it belongs to a code block or an HTML block, which no rule treats as prose.
	pub code: bool,
	/// The byte ranges formatters leave as written, sorted by start: inline code, inline HTML and link destinations.
	pub protected: Vec<Range<usize>>,
}

/// A table: the line of its header and the lines of its body; the delimiter row, right under the header, is not
/// listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
	/// The line of the header row.
	pub header: usize,
	/// The lines of the body rows, in order.
	pub body: Vec<usize>,
}

/// A heading: its level, 1 to 6, and its line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Heading {
	/// From 1 for `#` to 6 for `######`.
	pub level: u8,
	/// The line the heading starts on.
	pub line: usize,
}

/// A fenced code block: its info string, whose first word names the language, and the line it opens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fence {
	/// The info string, trimmed; empty when the fence names no language.
	pub info: String,
	/// The line of the opening fence.
	pub line: usize,
}

/// The marker of an item in an unordered list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bullet {
	/// The line of the item.
	pub line: usize,
	/// The byte of the line the marker is at.
	pub column: usize,
	/// The marker.
	pub marker: Marker,
	/// Whether the list stands right next to another list, which only a different marker keeps apart.
	pub adjacent: bool,
}

/// A marker `CommonMark` allows for an item of an unordered list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum Marker {
	/// `-`
	#[default]
	#[serde(rename = "-")]
	Dash,
	/// `*`
	#[serde(rename = "*")]
	Star,
	/// `+`
	#[serde(rename = "+")]
	Plus,
}

impl Marker {
	/// The marker `c` is, if it is one.
	#[must_use]
	pub fn from_char(c: char) -> Option<Self> {
		match c {
			'-' => Some(Self::Dash),
			'*' => Some(Self::Star),
			'+' => Some(Self::Plus),
			_ => None,
		}
	}

	/// The marker as written.
	#[must_use]
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Dash => "-",
			Self::Star => "*",
			Self::Plus => "+",
		}
	}
}

/// Something in a document with a destination: an inline link, an image or a reference definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
	/// The destination, with escapes and entities resolved.
	pub target: String,
	/// The line the destination is written on, or the line the link starts on when it writes none.
	pub line: usize,
	/// The bytes of `line` the destination is written in, inside any angle brackets; empty when it writes none.
	pub destination: Range<usize>,
	/// What it is, and what it shows.
	pub kind: LinkKind,
}

/// What a [`Link`] is, and what it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkKind {
	/// An inline link, `[text](target)`.
	Inline {
		/// Its text, without markup.
		text: String,
		/// Whether it has any content at all, images included.
		has_content: bool,
	},
	/// An image, `![alt](target)`.
	Image {
		/// Its alternative text, without markup.
		alt: String,
	},
	/// A reference definition, `[label]: target`.
	Definition,
}

/// A reference definition, `[label]: target`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
	/// The destination, with escapes and entities resolved.
	pub target: String,
	/// The line the definition starts on.
	pub line: usize,
}

/// A reference link: the normalized label it names and the line it starts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
	/// The label, lowercased, with its runs of white space collapsed to one space.
	pub label: String,
	/// The line the link starts on.
	pub line: usize,
}

impl Document {
	/// Parses `content`.
	#[must_use]
	pub fn parse(content: &str) -> Self {
		walker::parse(content)
	}
}

impl fmt::Display for Document {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, line) in self.lines.iter().enumerate() {
			if i > 0 {
				f.write_str("\n")?;
			}
			f.write_str(&line.text)?;
		}
		Ok(())
	}
}
