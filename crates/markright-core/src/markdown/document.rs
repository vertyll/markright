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
    /// Every block, containers before what they hold, in the order they start.
    pub blocks: Vec<Block>,
    /// The headings, in order.
    pub headings: Vec<Heading>,
    /// The fenced code blocks, in order.
    pub fences: Vec<Fence>,
    /// The items of every list, in order.
    pub items: Vec<ListItem>,
    /// The emphasis and strong emphasis, in the order they start.
    pub emphasis: Vec<Emphasis>,
    /// The URLs written as plain text, outside any link, in order.
    pub bare_urls: Vec<Span>,
    /// The code spans written on one line, their backticks included, in order.
    pub code_spans: Vec<Span>,
    /// The tables, in order.
    pub tables: Vec<Table>,
    /// The inline links and images, in the order they start, then the reference definitions.
    pub links: Vec<Link>,
    /// The reference definitions, by normalized label; the first of a repeated label wins, as it does in the parse.
    pub definitions: BTreeMap<String, Definition>,
    /// The reference definitions whose label an earlier definition already has, which the parse ignores.
    pub duplicate_definitions: Vec<Reference>,
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

/// Some bytes of one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The line.
    pub line: usize,
    /// The bytes of the line.
    pub columns: Range<usize>,
}

/// A block of the document: what it is, the lines it takes, and how many lists, list items and blockquotes it sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    /// What it is.
    pub kind: BlockKind,
    /// Its first line.
    pub first: usize,
    /// Its last line, without the blank lines after it.
    pub last: usize,
    /// How many lists, list items and blockquotes it sits in; 0 at the top of the document.
    pub depth: usize,
}

/// What a [`Block`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    /// A paragraph.
    Paragraph,
    /// A heading.
    Heading,
    /// A list.
    List,
    /// A blockquote.
    BlockQuote,
    /// A code block between fences.
    FencedCode,
    /// A code block indented by four spaces.
    IndentedCode,
    /// An HTML block.
    Html,
    /// A table.
    Table,
    /// A thematic break, a horizontal rule.
    Rule,
    /// A front matter block, in YAML between `---` or in TOML between `+++`, at the very start of the document.
    Metadata,
}

/// A heading: its level, 1 to 6, its line, its text and the anchor it defines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// From 1 for `#` to 6 for `######`.
    pub level: u8,
    /// The line the heading starts on.
    pub line: usize,
    /// Its text, without markup.
    pub text: String,
    /// The anchor a `#fragment` names it by, numbered when an earlier heading has the same.
    pub anchor: String,
    /// The line of its underline, `===` or `---`, when it is written as a setext heading.
    pub underline: Option<usize>,
}

/// A fenced code block: its info string, whose first word names the language, its fences and its lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fence {
    /// The info string, trimmed; empty when the fence names no language.
    pub info: String,
    /// The line of the opening fence.
    pub line: usize,
    /// The character of the fence: a backtick or a tilde.
    pub marker: char,
    /// How many of it the opening fence has.
    pub length: usize,
    /// The line of the closing fence; none when the block runs to the end of its container.
    pub closing: Option<usize>,
}

/// An item of a list: where its marker is, what it is, and how far its content stands from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItem {
    /// The line the item starts on.
    pub line: usize,
    /// The last line of the item, without the blank lines after it.
    pub last_line: usize,
    /// The byte of the line the marker is at.
    pub column: usize,
    /// The marker.
    pub marker: ItemMarker,
    /// How many spaces follow the marker on its line.
    pub spaces: usize,
    /// The list it belongs to, counted from 0 in the order the lists start.
    pub list: usize,
    /// Whether the list stands right next to another list, which only a different marker keeps apart.
    pub adjacent: bool,
}

/// The marker of a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemMarker {
    /// `-`, `*` or `+`, of an unordered list.
    Bullet(Marker),
    /// A number and its delimiter, `.` or `)`, of an ordered list.
    Number {
        /// The number as written.
        value: u64,
        /// How many digits are written, leading zeros included.
        digits: usize,
        /// `.` or `)`.
        delimiter: char,
    },
}

impl ItemMarker {
    /// How many bytes the marker takes.
    #[must_use]
    pub fn width(self) -> usize {
        match self {
            Self::Bullet(_) => 1,
            Self::Number { digits, .. } => digits + 1,
        }
    }
}

/// Emphasis, `*text*` or `_text_`, or strong emphasis, `**text**` or `__text__`: where its delimiters are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emphasis {
    /// Whether it is strong emphasis, with two-character delimiters.
    pub strong: bool,
    /// The opening delimiter.
    pub open: Span,
    /// The closing delimiter.
    pub close: Span,
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
    /// The bytes between the brackets of an inline link, when both brackets are on one line.
    pub label: Option<Span>,
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

    /// The block of `kind` that starts on `line` at the top of the document, outside lists and blockquotes.
    #[must_use]
    pub fn top_level(&self, kind: BlockKind, line: usize) -> Option<&Block> {
        self.blocks
            .iter()
            .find(|block| block.kind == kind && block.first == line && block.depth == 0)
    }

    /// Whether the 1-based `line` is blank, or outside the document: before its first line or after its last.
    #[must_use]
    pub fn is_blank(&self, line: usize) -> bool {
        line == 0 || self.lines.get(line - 1).is_none_or(|line| line.text.trim().is_empty())
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
