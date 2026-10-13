//! The repository's documents as pages of the site: each document `CONTENTS.md` lists, at its place in the
//! documentation, with its links pointed at the pages they lead to on the site, or at the repository.

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::Path;

use markright::markdown::{self, Document};
use markright::{formatters, linters};

use super::diagrams;
use super::markup::{front_matter, string};
use super::rules::{self, Kind};

/// Where documents link to for files that are not pages of the site.
const REPOSITORY: &str = "https://github.com/vertyll/markright";
/// The index that lists every document of the repository, and so every document of the site.
const INDEX: &str = "CONTENTS.md";
/// The directory of the documents on how markright reads a document, which form a section of their own.
const MECHANISMS: &str = "docs/mechanisms/";

/// A document's place on the site: its page in the content, its title when it is not the document's own, and its
/// position in the documentation's navigation.
#[derive(Clone)]
struct Place {
	source: Cow<'static, str>,
	target: Cow<'static, str>,
	title: Option<&'static str>,
	weight: usize,
}

/// The documents with a place of their own; the mechanisms follow them in a section.
static PLACES: [Place; 7] = [
	place("README.md", "docs/_index.md", Some("Getting started"), 1),
	place("docs/rules.md", "docs/rules.md", None, 2),
	place("docs/configuration.md", "docs/configuration.md", None, 5),
	place("docs/architecture.md", "docs/architecture.md", None, 6),
	place("docs/development.md", "docs/development.md", None, 8),
	place("GLOSSARY.md", "docs/glossary.md", None, 9),
	place("STANDARDS.md", "docs/standards.md", None, 10),
];

const fn place(source: &'static str, target: &'static str, title: Option<&'static str>, weight: usize) -> Place {
	Place {
		source: Cow::Borrowed(source),
		target: Cow::Borrowed(target),
		title,
		weight,
	}
}

/// A section of the documentation the generator writes: its page, title, position, introduction and the template of
/// its own page and of its pages.
pub(super) struct Section {
	pub(super) target: &'static str,
	pub(super) title: &'static str,
	pub(super) weight: usize,
	pub(super) intro: &'static str,
	pub(super) template: &'static str,
	pub(super) page_template: &'static str,
}

const MECHANISMS_SECTION: Section = Section {
	target: "docs/mechanisms/_index.md",
	title: "Mechanisms",
	weight: 7,
	intro: "How markright reads a document: which lines are code, how anchors are made, and what formatters leave \
	        alone.",
	template: "docs-section.html",
	page_template: "docs-page.html",
};

/// The page a document's page is the section of: every document lives under `docs/`.
const DOCUMENTATION: &str = "docs/_index.md";

/// A card that leads to a page a reader is likely to want next: its title, a line on what is there and the page in
/// the content.
struct Card {
	title: &'static str,
	subtitle: &'static str,
	page: &'static str,
}

/// The cards that close the getting started page.
const NEXT: [Card; 4] = [
	Card {
		title: "Linters",
		subtitle: "What markright reports, with an example of each",
		page: "docs/linters/_index.md",
	},
	Card {
		title: "Formatters",
		subtitle: "What markright fmt fixes, before and after",
		page: "docs/formatters/_index.md",
	},
	Card {
		title: "Configuration",
		subtitle: "Choosing rules, their settings and exclusions",
		page: "docs/configuration.md",
	},
	Card {
		title: "Playground",
		subtitle: "Try markright on a document, in the browser",
		page: "playground.md",
	},
];

/// Every page that comes from a document, and the mechanisms' section, by path in the content.
pub(super) fn pages(root: &Path) -> io::Result<BTreeMap<String, String>> {
	let listed = listed(root)?;
	let mechanisms = listed
		.iter()
		.filter(|document| document.starts_with(MECHANISMS))
		.zip(1..)
		.map(|(document, weight)| Place {
			source: Cow::Owned(document.clone()),
			target: Cow::Owned(document.clone()),
			title: None,
			weight,
		});
	let places: Vec<Place> = PLACES.iter().cloned().chain(mechanisms).collect();
	if let Some(lost) = listed
		.iter()
		.find(|document| !places.iter().any(|place| place.source == document.as_str()))
	{
		return Err(io::Error::other(format!(
			"{lost} is listed in {INDEX} but has no place on the site"
		)));
	}
	let linker = Linker {
		root,
		summaries: summaries(root)?,
		targets: places
			.iter()
			.map(|place| (place.source.to_string(), place.target.to_string()))
			.collect(),
	};
	let mut pages = BTreeMap::new();
	for place in &places {
		let text = fs::read_to_string(root.join(place.source.as_ref()))?;
		pages.insert(place.target.to_string(), linker.page(place, &text)?);
	}
	pages.insert(MECHANISMS_SECTION.target.to_owned(), section_page(&MECHANISMS_SECTION));
	Ok(pages)
}

/// The page of a section the generator writes: its front matter and its introduction.
pub(super) fn section_page(section: &Section) -> String {
	let fields = [
		("title", string(section.title)),
		("weight", section.weight.to_string()),
		("sort_by", string("weight")),
		("template", string(section.template)),
		("page_template", string(section.page_template)),
	];
	format!("{}{}\n", front_matter(&fields, &[]), section.intro)
}

/// The documents `CONTENTS.md` links to, each once, in order.
fn listed(root: &Path) -> io::Result<Vec<String>> {
	let index = Document::parse(&fs::read_to_string(root.join(INDEX))?);
	let mut documents: Vec<String> = Vec::new();
	for link in &index.links {
		let file = link.target.split('#').next().unwrap_or_default();
		if Path::new(file).extension().is_some_and(|extension| extension == "md") && !is_external(file) {
			let file = normalize(file);
			if !documents.contains(&file) {
				documents.push(file);
			}
		}
	}
	Ok(documents)
}

/// What each document covers, by path: the last cell of its row in the tables of `CONTENTS.md`, as text.
fn summaries(root: &Path) -> io::Result<HashMap<String, String>> {
	let index = Document::parse(&fs::read_to_string(root.join(INDEX))?);
	let mut summaries = HashMap::new();
	for &row in index.tables.iter().flat_map(|table| &table.body) {
		let Some(link) = index.links.iter().find(|link| link.line == row) else {
			continue;
		};
		let text = &index.lines[row - 1].text;
		let covers = text
			.trim()
			.trim_end_matches('|')
			.rsplit('|')
			.next()
			.unwrap_or_default()
			.trim();
		summaries.insert(normalize(&link.target), markdown::plain(covers));
	}
	Ok(summaries)
}

/// Points the links of documents at their pages: `targets` holds the page of every document on the site.
struct Linker<'a> {
	root: &'a Path,
	/// What each document covers, as the index says, by path.
	summaries: HashMap<String, String>,
	targets: HashMap<String, String>,
}

impl Linker<'_> {
	/// The page of the document at `place`, whose text is `text`: front matter, then the body after its title.
	fn page(&self, place: &Place, text: &str) -> io::Result<String> {
		let (source, target) = (place.source.as_ref(), place.target.as_ref());
		let document = Document::parse(text);
		let mut lines = self.linked(&document, source);
		link_rule_names(&document, &mut lines);
		diagrams::draw(&document, &mut lines, target)?;
		for heading in &document.headings {
			let _ = write!(lines[heading.line - 1], " {{#{}}}", heading.anchor);
		}
		let (own_title, body) = title_and_body(&document, &lines);
		let title = place
			.title
			.map_or_else(|| own_title.unwrap_or_else(|| source.to_owned()), str::to_owned);
		let mut fields = vec![("title", string(&title)), ("weight", place.weight.to_string())];
		if let Some(summary) = self.summaries.get(source) {
			fields.push(("description", string(summary)));
		}
		let mut extra = Vec::new();
		if target.ends_with("/_index.md") {
			fields.push(("sort_by", string("weight")));
			fields.push(("template", string("docs-section.html")));
			fields.push(("page_template", string("docs-page.html")));
		} else {
			fields.push(("template", string("docs-page.html")));
		}
		if target == DOCUMENTATION {
			extra.push(("navigation", navigation()));
			extra.push(("next", next_cards()));
		}
		Ok(format!(
			"{}{}\n",
			front_matter(&fields, &extra),
			body.trim_matches('\n')
		))
	}

	/// The lines of `document`, the document at `source`, with the destination of every link, image and reference
	/// definition pointed at its page on the site or at the repository. The parser says where each destination is
	/// written, so code and text that only looks like a link stay as written.
	fn linked(&self, document: &Document, source: &str) -> Vec<String> {
		let mut lines: Vec<String> = document.lines.iter().map(|line| line.text.clone()).collect();
		let mut written: Vec<_> = document
			.links
			.iter()
			.filter(|link| !link.destination.is_empty())
			.collect();
		// Right to left within a line, so each edit leaves the bytes of the ones before it where they were.
		written.sort_by_key(|link| (link.line, Reverse(link.destination.start)));
		for link in written {
			let line = &mut lines[link.line - 1];
			line.replace_range(link.destination.clone(), &self.resolve(&link.target, source));
		}
		lines
	}

	/// Where `target`, a link of the document at `source`, leads on the site: a page as Zola names it, `@/` and its
	/// path in the content, which Zola checks when it builds; a file of the repository; or `target` itself when it
	/// leaves the repository or stays on the page.
	fn resolve(&self, target: &str, source: &str) -> String {
		if is_external(target) || target.starts_with('#') {
			return target.to_owned();
		}
		let (file, anchor) = match target.split_once('#') {
			Some((file, fragment)) => (file, format!("#{fragment}")),
			None => (target, String::new()),
		};
		let directory = source.rsplit_once('/').map_or("", |(directory, _)| directory);
		let path = normalize(&format!("{directory}/{file}"));
		if let Some(page) = self.targets.get(&path) {
			return format!("@/{page}{anchor}");
		}
		let kind = if self.root.join(&path).is_dir() { "tree" } else { "blob" };
		format!("{REPOSITORY}/{kind}/main/{path}{anchor}")
	}
}

/// The title of `document`, its first heading when that is of level 1, and the body after it, from `lines`, without
/// its documentation section, which the site's navigation replaces.
fn title_and_body(document: &Document, lines: &[String]) -> (Option<String>, String) {
	let title = document.headings.first().filter(|heading| heading.level == 1);
	let start = title.map_or(0, |heading| heading.line);
	let mut body = Vec::new();
	let mut skipping = false;
	for (index, line) in lines.iter().enumerate().skip(start) {
		let section = document
			.headings
			.iter()
			.find(|heading| heading.line == index + 1 && heading.level == 2);
		if let Some(heading) = section {
			skipping = heading_text(&document.lines[heading.line - 1].text) == "Documentation";
		}
		if !skipping {
			body.push(line.as_str());
		}
	}
	let title = title.map(|heading| heading_text(&document.lines[heading.line - 1].text));
	(title, body.join("\n"))
}

/// Turns the rule named in the first cell of a table row into a link to its page. The parser says which lines are
/// table rows, so a line that only looks like one stays as written.
fn link_rule_names(document: &Document, lines: &mut [String]) {
	for table in &document.tables {
		for &row in &table.body {
			let line = &mut lines[row - 1];
			let Some(cell) = line
				.trim_start()
				.strip_prefix('|')
				.and_then(|rest| rest.split('|').next())
			else {
				continue;
			};
			let name = cell.trim().trim_matches('`');
			let kind = if linters::names().contains(&name) {
				Kind::Linter
			} else if formatters::names().contains(&name) {
				Kind::Formatter
			} else {
				continue;
			};
			let code = format!("`{name}`");
			*line = line.replacen(&code, &format!("[{code}](@/{})", rules::page(kind, name)), 1);
		}
	}
}

/// The text of an ATX heading's line, without its markers and markup.
fn heading_text(line: &str) -> String {
	markdown::plain(line.trim_start_matches('#').trim())
}

/// The order of the documentation's navigation: each page or section by its path in the content.
fn navigation() -> String {
	let mut entries: Vec<(usize, &str)> = PLACES
		.iter()
		.map(|place| (place.weight, place.target.as_ref()))
		.collect();
	entries.extend(rules::SECTIONS.iter().map(|section| (section.weight, section.target)));
	entries.push((MECHANISMS_SECTION.weight, MECHANISMS_SECTION.target));
	entries.sort_unstable();
	let paths: Vec<String> = entries.iter().map(|(_, path)| string(path)).collect();
	format!("[{}]", paths.join(", "))
}

/// The cards that close the getting started page, as a TOML array.
fn next_cards() -> String {
	let cards: Vec<String> = NEXT
		.iter()
		.map(|card| {
			format!(
				"{{ title = {}, subtitle = {}, page = {} }}",
				string(card.title),
				string(card.subtitle),
				string(card.page)
			)
		})
		.collect();
	format!("[\n\t{},\n]", cards.join(",\n\t"))
}

/// Whether a link target leaves the repository: a URL with a scheme, or a mail address.
fn is_external(target: &str) -> bool {
	target.contains("://") || target.starts_with("mailto:")
}

/// `path` with its `.` and `..` segments resolved, and no leading slash.
fn normalize(path: &str) -> String {
	let mut parts: Vec<&str> = Vec::new();
	for part in path.split('/') {
		match part {
			"" | "." => {}
			".." => {
				parts.pop();
			}
			_ => parts.push(part),
		}
	}
	parts.join("/")
}
