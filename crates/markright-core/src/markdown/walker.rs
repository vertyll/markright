//! Builds a [`Document`] from the events of the parser and the byte ranges it gives for each.

use std::collections::{BTreeMap, HashSet};
use std::ops::Range;
use std::sync::LazyLock;

use pulldown_cmark::{BrokenLink, CodeBlockKind, Event, LinkType, Options, Parser, Tag, TagEnd};
use regex::Regex;

use super::anchors::Anchors;
use super::lines::LineIndex;
use super::{
	Block, BlockKind, Definition, Document, Emphasis, Fence, Heading, ItemMarker, Line, Link, LinkKind, ListItem,
	Marker, Reference, Span, Table, destination, normalize_label,
};

/// The extensions GFM adds that change what a document holds: tables, strikethrough and task lists; and front matter,
/// which no rule reads as Markdown. Alerts are left to the alerts linter, which needs to see their tags.
const OPTIONS: Options = Options::ENABLE_TABLES
	.union(Options::ENABLE_STRIKETHROUGH)
	.union(Options::ENABLE_TASKLISTS)
	.union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
	.union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

/// A URL with a scheme, as GFM finds one in plain text, before its trailing punctuation is cut off.
static URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://[^\s<>`]+").expect("the URL pattern is valid"));

/// Parses `content` into a document.
pub(super) fn parse(content: &str) -> Document {
	let index = LineIndex::new(content);
	let mut broken = Vec::new();
	let mut on_broken = |link: BrokenLink<'_>| {
		if matches!(link.link_type, LinkType::Reference | LinkType::Collapsed) && !link.reference.starts_with('^') {
			broken.push(Reference {
				label: normalize_label(&link.reference),
				line: index.line(link.span.start) + 1,
			});
		}
		None
	};
	let parser = Parser::new_with_broken_link_callback(content, OPTIONS, Some(&mut on_broken));
	let mut definitions: Vec<Written> = parser
		.reference_definitions()
		.iter()
		.filter(|(label, _)| !label.starts_with('^'))
		.map(|(label, def)| Written {
			label: normalize_label(label),
			target: def.dest.to_string(),
			span: def.span.clone(),
		})
		.collect();
	definitions.sort_by_key(|definition| definition.span.start);
	let mut walker = Walker::new(content, &index);
	for (event, range) in parser.into_offset_iter() {
		walker.event(event, range);
	}
	walker.into_document(&definitions, broken)
}

/// A reference definition as the parser gives it: its label, its target and where it is written.
struct Written {
	label: String,
	target: String,
	span: Range<usize>,
}

/// For an open block, what the child closed last was: a list, by its index, or something else.
#[derive(Default, Clone, Copy)]
struct Sibling(Option<usize>);

/// An open list: whether it is ordered, and its index among the lists.
struct OpenList {
	ordered: bool,
	index: usize,
}

/// Text being gathered for a heading, a link or an image: the plain text, whether anything at all is inside, how many
/// images were open when it started, so the text of an image inside it is left out, and where its content ends.
struct Gathering {
	purpose: Purpose,
	text: String,
	has_content: bool,
	images: usize,
	last_end: usize,
}

/// What gathered text is for.
enum Purpose {
	Heading {
		level: u8,
		line: usize,
		underline: Option<usize>,
	},
	Link {
		target: String,
		image: bool,
		start: usize,
		text_start: usize,
	},
	/// A reference link or an autolink, which the links do not list; it is closed like any other.
	Unlisted,
}

/// Walks the parser's events and records, line by line, what the document holds.
struct Walker<'a> {
	source: &'a str,
	index: &'a LineIndex,
	anchors: Anchors,
	code: Vec<bool>,
	protected: Vec<Vec<Range<usize>>>,
	blocks: Vec<Block>,
	/// How many lists, list items and blockquotes are open.
	containers: usize,
	/// Whether the walk is inside a code block, an HTML block or front matter, whose text is not prose.
	literal: bool,
	headings: Vec<Heading>,
	fences: Vec<Fence>,
	items: Vec<ListItem>,
	emphasis: Vec<Emphasis>,
	/// The stretches of prose outside links, each as long as the text events that follow one another make it.
	prose: Vec<Range<usize>>,
	tables: Vec<Table>,
	/// Each inline link and image, with the offset it starts at, as it closes.
	links: Vec<(usize, Link)>,
	used_labels: HashSet<String>,
	lazy_lines: Vec<usize>,
	unclosed_fence: Option<usize>,
	/// Where each list is in the source, by index.
	lists: Vec<Range<usize>>,
	/// The lists that follow another list with no block between them, as the indexes of both.
	neighbors: Vec<(usize, usize)>,
	/// One entry per open block, the document first.
	siblings: Vec<Sibling>,
	open_lists: Vec<OpenList>,
	blockquotes: usize,
	images: usize,
	texts: Vec<Gathering>,
}

impl<'a> Walker<'a> {
	fn new(source: &'a str, index: &'a LineIndex) -> Self {
		Self {
			source,
			index,
			anchors: Anchors::default(),
			code: vec![false; index.count()],
			protected: vec![Vec::new(); index.count()],
			blocks: Vec::new(),
			containers: 0,
			literal: false,
			headings: Vec::new(),
			fences: Vec::new(),
			items: Vec::new(),
			emphasis: Vec::new(),
			prose: Vec::new(),
			tables: Vec::new(),
			links: Vec::new(),
			used_labels: HashSet::new(),
			lazy_lines: Vec::new(),
			unclosed_fence: None,
			lists: Vec::new(),
			neighbors: Vec::new(),
			siblings: vec![Sibling::default()],
			open_lists: Vec::new(),
			blockquotes: 0,
			images: 0,
			texts: Vec::new(),
		}
	}

	fn event(&mut self, event: Event<'_>, range: Range<usize>) {
		match event {
			Event::Start(tag) => self.start(tag, range),
			Event::End(end) => self.end(end, range),
			Event::Text(text) => {
				self.text(&text, &range);
				self.prose(&range);
			}
			Event::Code(code) => {
				self.text(&code, &range);
				self.protect_code(&range);
			}
			Event::InlineHtml(html) => {
				self.touch(&range);
				self.anchors.html(&html);
				self.protect(range);
			}
			Event::Html(html) => self.anchors.html(&html),
			Event::SoftBreak | Event::HardBreak => self.text(" ", &range),
			Event::Rule => {
				self.sibling(None);
				self.block(BlockKind::Rule, &range);
			}
			_ => self.touch(&range),
		}
	}

	fn start(&mut self, tag: Tag<'_>, range: Range<usize>) {
		let list = matches!(tag, Tag::List(_)).then(|| self.lists.len());
		if let Some(index) = list {
			self.lists.push(range.clone());
			if let Some(previous) = self.siblings.last().and_then(|sibling| sibling.0) {
				self.neighbors.push((previous, index));
			}
		}
		self.sibling(None);
		self.siblings.push(Sibling::default());
		match tag {
			Tag::Heading { level, .. } => {
				self.block(BlockKind::Heading, &range);
				let (start, end) = self.span_lines(&range);
				let underlined = end > start && is_underline(self.index.text(self.source, end));
				let purpose = Purpose::Heading {
					level: level as u8,
					line: start + 1,
					underline: underlined.then_some(end + 1),
				};
				self.texts.push(Gathering::new(purpose, self.images, range.start));
			}
			Tag::Paragraph => {
				self.block(BlockKind::Paragraph, &range);
				if self.blockquotes > 0 {
					self.lazy_lines(&range);
				}
			}
			Tag::CodeBlock(kind) => {
				let block = match kind {
					CodeBlockKind::Fenced(_) => BlockKind::FencedCode,
					CodeBlockKind::Indented => BlockKind::IndentedCode,
				};
				self.block(block, &range);
				self.literal = true;
				self.code_block(&kind, &range);
			}
			Tag::HtmlBlock => {
				self.block(BlockKind::Html, &range);
				self.literal = true;
				self.mark_code(&range);
			}
			Tag::MetadataBlock(_) => {
				self.block(BlockKind::Metadata, &range);
				self.literal = true;
				self.mark_code(&range);
			}
			Tag::Table(_) => self.block(BlockKind::Table, &range),
			Tag::BlockQuote(_) => {
				self.block(BlockKind::BlockQuote, &range);
				self.blockquotes += 1;
				self.containers += 1;
			}
			Tag::List(first) => {
				self.block(BlockKind::List, &range);
				self.containers += 1;
				self.open_lists.push(OpenList {
					ordered: first.is_some(),
					index: list.unwrap_or_default(),
				});
			}
			Tag::Item => {
				self.item(&range);
				self.containers += 1;
			}
			Tag::Emphasis => self.emphasis(&range, false),
			Tag::Strong => self.emphasis(&range, true),
			Tag::TableHead => {
				self.tables.push(Table {
					header: self.index.line(range.start) + 1,
					body: Vec::new(),
				});
			}
			Tag::TableRow => {
				if let Some(table) = self.tables.last_mut() {
					table.body.push(self.index.line(range.start) + 1);
				}
			}
			Tag::Link {
				link_type,
				dest_url,
				id,
				..
			} => self.link(link_type, &dest_url, &id, range.start, false),
			Tag::Image {
				link_type,
				dest_url,
				id,
				..
			} => {
				self.touch(&range);
				self.images += 1;
				self.link(link_type, &dest_url, &id, range.start, true);
			}
			_ => {}
		}
	}

	fn end(&mut self, end: TagEnd, range: Range<usize>) {
		self.siblings.pop();
		let list = match end {
			TagEnd::List(_) => self.open_lists.pop().map(|list| list.index),
			_ => None,
		};
		self.sibling(list);
		match end {
			TagEnd::Heading(_) | TagEnd::Link => self.finish_text(),
			TagEnd::BlockQuote(_) => {
				self.blockquotes -= 1;
				self.containers -= 1;
			}
			TagEnd::List(_) | TagEnd::Item => self.containers -= 1,
			TagEnd::CodeBlock | TagEnd::HtmlBlock | TagEnd::MetadataBlock(_) => self.literal = false,
			TagEnd::Image => {
				self.images -= 1;
				self.finish_text();
				self.touch(&range);
			}
			_ => {}
		}
	}

	fn sibling(&mut self, list: Option<usize>) {
		if let Some(top) = self.siblings.last_mut() {
			*top = Sibling(list);
		}
	}

	/// Records a block of `kind` over `range`, as deep as the containers open around it.
	fn block(&mut self, kind: BlockKind, range: &Range<usize>) {
		let (first, last) = self.span_lines(range);
		self.blocks.push(Block {
			kind,
			first: first + 1,
			last: last + 1,
			depth: self.containers,
		});
	}

	/// The 0-based lines `range` covers, from the one it starts on to the one its last character other than white
	/// space is on: the parser counts the blank lines after a block as part of it.
	fn span_lines(&self, range: &Range<usize>) -> (usize, usize) {
		let content = self.source[range.clone()].trim_end().len();
		let first = self.index.line(range.start);
		(
			first,
			self.index.line(range.start + content.saturating_sub(1)).max(first),
		)
	}

	/// The bytes `range` covers as a span of one line, or nothing when it runs over more than one.
	fn span(&self, range: Range<usize>) -> Option<Span> {
		let line = self.index.line(range.start);
		(range.is_empty() || self.index.line(range.end - 1) == line).then(|| {
			let column = self.index.column(range.start);
			Span {
				line: line + 1,
				columns: column..column + range.len(),
			}
		})
	}

	/// Records emphasis over `range`, whose delimiters are one character long, or two for strong emphasis.
	fn emphasis(&mut self, range: &Range<usize>, strong: bool) {
		let width = if strong { 2 } else { 1 };
		let delimited = self.source[range.clone()].starts_with(['*', '_']) && range.len() >= 2 * width;
		let open = delimited.then(|| self.span(range.start..range.start + width)).flatten();
		let close = delimited.then(|| self.span(range.end - width..range.end)).flatten();
		if let (Some(open), Some(close)) = (open, close) {
			self.emphasis.push(Emphasis { strong, open, close });
		}
	}

	/// Records the text at `range` as prose, unless it is the text of a link or of a literal block.
	fn prose(&mut self, range: &Range<usize>) {
		let in_link = self
			.texts
			.iter()
			.any(|gathering| matches!(gathering.purpose, Purpose::Link { .. } | Purpose::Unlisted));
		if self.literal || in_link {
			return;
		}
		match self.prose.last_mut() {
			Some(last) if last.end == range.start => last.end = range.end,
			_ => self.prose.push(range.clone()),
		}
	}

	fn text(&mut self, text: &str, range: &Range<usize>) {
		let images = self.images;
		for gathering in &mut self.texts {
			gathering.has_content = true;
			gathering.last_end = gathering.last_end.max(range.end);
			if gathering.images == images {
				gathering.text.push_str(text);
			}
		}
	}

	/// Marks the open links and images as having content, without adding text.
	fn touch(&mut self, range: &Range<usize>) {
		for gathering in &mut self.texts {
			gathering.has_content = true;
			gathering.last_end = gathering.last_end.max(range.end);
		}
	}

	fn finish_text(&mut self) {
		let Some(gathering) = self.texts.pop() else { return };
		match gathering.purpose {
			Purpose::Heading { level, line, underline } => {
				let anchor = self.anchors.heading(&gathering.text);
				self.headings.push(Heading {
					level,
					line,
					text: gathering.text.trim().to_owned(),
					anchor,
					underline,
				});
			}
			Purpose::Link {
				target,
				image,
				start,
				text_start,
			} => {
				let text_end = gathering.last_end.max(text_start);
				let written = destination::inline(self.source, text_end);
				let (line, destination) = self.place(written.clone(), start);
				let label = destination::closing_bracket(self.source, text_end)
					.filter(|_| !image)
					.and_then(|close| self.span(text_start..close));
				if let Some(written) = written {
					self.protect(written);
				}
				let text = gathering.text.trim().to_owned();
				let kind = if image {
					LinkKind::Image { alt: text }
				} else {
					LinkKind::Inline {
						text,
						has_content: gathering.has_content,
					}
				};
				let found = Link {
					target,
					line,
					destination,
					label,
					kind,
				};
				self.links.push((start, found));
			}
			Purpose::Unlisted => {}
		}
	}

	/// The line a destination is written on and the bytes of that line it takes, or, when it writes none, the line
	/// `start` is on and no bytes.
	fn place(&self, written: Option<Range<usize>>, start: usize) -> (usize, Range<usize>) {
		match written {
			Some(written) => {
				let column = self.index.column(written.start);
				(self.index.line(written.start) + 1, column..column + written.len())
			}
			None => (self.index.line(start) + 1, 0..0),
		}
	}

	fn link(&mut self, link_type: LinkType, dest: &str, id: &str, start: usize, image: bool) {
		let purpose = match link_type {
			LinkType::Inline => {
				let text_start = start + if image { 2 } else { 1 };
				Purpose::Link {
					target: dest.to_owned(),
					image,
					start,
					text_start,
				}
			}
			LinkType::Reference | LinkType::Collapsed | LinkType::Shortcut => {
				self.used_labels.insert(normalize_label(id));
				Purpose::Unlisted
			}
			_ => Purpose::Unlisted,
		};
		let text_start = match purpose {
			Purpose::Link { text_start, .. } => text_start,
			_ => start,
		};
		self.texts.push(Gathering::new(purpose, self.images, text_start));
	}

	/// Records the item at `range`: its marker, the spaces after it and the lines it takes.
	fn item(&mut self, range: &Range<usize>) {
		let Some(&OpenList { ordered, index }) = self.open_lists.last() else {
			return;
		};
		let text = &self.source[range.start..];
		let marker = if ordered {
			let digits = text.bytes().take_while(u8::is_ascii_digit).count();
			let delimiter = text[digits..].chars().next();
			match (text[..digits].parse(), delimiter) {
				(Ok(value), Some(delimiter @ ('.' | ')'))) => ItemMarker::Number {
					value,
					digits,
					delimiter,
				},
				_ => return,
			}
		} else {
			match text.chars().next().and_then(Marker::from_char) {
				Some(marker) => ItemMarker::Bullet(marker),
				None => return,
			}
		};
		let (first, last) = self.span_lines(range);
		self.items.push(ListItem {
			line: first + 1,
			last_line: last + 1,
			column: self.index.column(range.start),
			marker,
			spaces: text[marker.width()..].bytes().take_while(|&b| b == b' ').count(),
			list: index,
			adjacent: false,
		});
	}

	fn code_block(&mut self, kind: &CodeBlockKind<'_>, range: &Range<usize>) {
		self.mark_code(range);
		let CodeBlockKind::Fenced(info) = kind else { return };
		let lines = self.index.lines(range);
		let (open, last) = (*lines.start(), *lines.end());
		let text = |line| self.index.text(self.source, line);
		let opening = strip_containers(text(open));
		let marker = opening.chars().next().unwrap_or('`');
		let closed = last > open && closes_fence(text(open), text(last));
		self.fences.push(Fence {
			info: info.trim().to_owned(),
			line: open + 1,
			marker,
			length: opening.chars().take_while(|&c| c == marker).count(),
			closing: closed.then_some(last + 1),
		});
		if !closed && self.unclosed_fence.is_none() {
			self.unclosed_fence = Some(open + 1);
		}
	}

	fn mark_code(&mut self, range: &Range<usize>) {
		for line in self.index.lines(range) {
			self.code[line] = true;
		}
	}

	fn lazy_lines(&mut self, range: &Range<usize>) {
		for line in self.index.lines(range) {
			if !self.index.text(self.source, line).trim().starts_with('>') {
				self.lazy_lines.push(line + 1);
			}
		}
	}

	/// Protects the content of a code span, between its backticks.
	fn protect_code(&mut self, range: &Range<usize>) {
		let span = &self.source[range.clone()];
		let ticks = span.bytes().take_while(|&b| b == b'`').count();
		if span.len() > 2 * ticks {
			self.protect(range.start + ticks..range.end - ticks);
		}
	}

	/// Records `range` as protected, split at the line breaks it spans.
	fn protect(&mut self, range: Range<usize>) {
		let mut start = range.start;
		while start < range.end {
			let line = self.index.line(start);
			let end = range.end.min(self.index.end(line));
			let column = self.index.column(start);
			if start < end {
				self.protected[line].push(column..column + end - start);
			}
			start = end.max(self.index.end(line) + 1);
		}
	}

	/// The document the walk found, with the reference definitions, which the parser gives apart from its events,
	/// and the reference links nothing defines.
	fn into_document(mut self, written: &[Written], undefined_references: Vec<Reference>) -> Document {
		let mut definitions = BTreeMap::new();
		let mut definition_links = Vec::with_capacity(written.len());
		for Written { label, target, span } in written {
			let destination = destination::definition(self.source, span);
			let (line, columns) = self.place(destination.clone(), span.start);
			if let Some(destination) = destination {
				self.protect(destination);
			}
			let entry = Link {
				target: target.clone(),
				line,
				destination: columns,
				label: None,
				kind: LinkKind::Definition,
			};
			definition_links.push(entry);
			let definition = Definition {
				target: target.clone(),
				line: self.index.line(span.start) + 1,
			};
			definitions.entry(label.clone()).or_insert(definition);
		}
		let spans: Vec<&Range<usize>> = written.iter().map(|definition| &definition.span).collect();
		let items = self.mark_adjacent(&spans);
		let bare_urls = self.bare_urls();
		self.links.sort_by_key(|(start, _)| *start);
		let all_links = self
			.links
			.into_iter()
			.map(|(_, link)| link)
			.chain(definition_links)
			.collect();
		let lines = self
			.source
			.split('\n')
			.zip(self.code)
			.zip(self.protected)
			.map(|((text, code), mut protected)| {
				protected.sort_by_key(|span| span.start);
				Line {
					text: text.to_owned(),
					code,
					protected,
				}
			})
			.collect();
		Document {
			lines,
			anchors: self.anchors.into_set(),
			blocks: self.blocks,
			headings: self.headings,
			fences: self.fences,
			items,
			emphasis: self.emphasis,
			bare_urls,
			tables: self.tables,
			links: all_links,
			definitions,
			used_labels: self.used_labels,
			undefined_references,
			lazy_lines: self.lazy_lines,
			unclosed_fence: self.unclosed_fence,
		}
	}

	/// The URLs written as plain text in the prose, each without the punctuation that ends its sentence, as GFM finds
	/// them.
	fn bare_urls(&self) -> Vec<Span> {
		let mut urls = Vec::new();
		for range in &self.prose {
			for found in URL.find_iter(&self.source[range.clone()]) {
				let end = range.start + found.start() + url_length(found.as_str());
				if let Some(span) = self.span(range.start + found.start()..end) {
					urls.push(span);
				}
			}
		}
		urls
	}

	/// The items, each marked when its list stands next to another list. The parser gives no event for a reference
	/// definition, so two lists with only definitions between them look like neighbors; the definitions keep them
	/// apart.
	fn mark_adjacent(&mut self, definitions: &[&Range<usize>]) -> Vec<ListItem> {
		let mut adjacent = vec![false; self.lists.len()];
		for &(previous, next) in &self.neighbors {
			let between = self.lists[previous].end..self.lists[next].start;
			if !definitions.iter().any(|span| between.contains(&span.start)) {
				adjacent[previous] = true;
				adjacent[next] = true;
			}
		}
		std::mem::take(&mut self.items)
			.into_iter()
			.map(|item| ListItem {
				adjacent: adjacent[item.list],
				..item
			})
			.collect()
	}
}

impl Gathering {
	fn new(purpose: Purpose, images: usize, start: usize) -> Self {
		Self {
			purpose,
			text: String::new(),
			has_content: false,
			images,
			last_end: start,
		}
	}
}

/// Whether `line` closes the fenced code block `opening` opens: a run of at least as many of the same fence
/// characters and nothing else, inside the same containers.
fn closes_fence(opening: &str, line: &str) -> bool {
	let open = strip_containers(opening);
	let Some(fence) = open.chars().next() else { return false };
	let marker = &open[..open.len() - open.trim_start_matches(fence).len()];
	let closing = strip_containers(line);
	!closing.is_empty() && closing.starts_with(marker) && closing.trim_matches(fence).is_empty()
}

/// `line` without its blockquote markers and the space around them.
fn strip_containers(line: &str) -> &str {
	let mut inner = line.trim();
	while let Some(rest) = inner.strip_prefix('>') {
		inner = rest.trim();
	}
	inner
}

/// Whether `line` underlines a setext heading: `=` or `-` and nothing else but spaces around them.
fn is_underline(line: &str) -> bool {
	let line = line.trim();
	!line.is_empty() && (line.bytes().all(|b| b == b'=') || line.bytes().all(|b| b == b'-'))
}

/// How much of `url` is the URL: GFM leaves out the punctuation that ends a sentence, and a `)` it does not open.
fn url_length(url: &str) -> usize {
	let mut end = url.len();
	loop {
		let trimmed = url[..end].trim_end_matches(['.', ',', ':', ';', '!', '?', '*', '_', '~', '\'', '"']);
		let opened = trimmed.matches('(').count();
		end = if trimmed.ends_with(')') && trimmed.matches(')').count() > opened {
			trimmed.len() - 1
		} else {
			trimmed.len()
		};
		if end == trimmed.len() {
			return end;
		}
	}
}
