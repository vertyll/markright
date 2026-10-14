use std::collections::{BTreeMap, HashMap};
use std::io;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::settings::{Index, LineLength};
use super::{
	ALL, Context, Kind, Linter, Repo, Settings, alerts, anchors, blank_line_in_blockquote, blockquotes,
	code_block_style, duplicate_definitions, duplicate_headings, emphasis_as_heading, fence_language, fences,
	first_heading, heading_increment, heading_punctuation, heading_without_space, image_alt, index, line_length,
	link_text, links, list_indent, readme_links, reversed_links, single_title, table_columns, unused_definitions,
};
use crate::fs::{Files, is_markdown};
use crate::markdown::Document;

fn files(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
	entries
		.iter()
		.map(|&(path, content)| (path.to_owned(), content.to_owned()))
		.collect()
}

fn repo(files: &BTreeMap<String, String>) -> Repo<'_> {
	let docs = files
		.iter()
		.filter(|(path, _)| is_markdown(path))
		.map(|(p, c)| (p.clone(), Document::parse(c)));
	Repo::new(files, docs.collect())
}

/// What `linter` finds in the document at `path`, or in the whole repository, with `settings` and the documents
/// `excluded` names left out, each as `path:line: message`, sorted.
fn found_excluding(
	linter: Linter,
	repo: &Repo<'_>,
	path: &str,
	settings: &Settings,
	excluded: &(dyn Fn(&str) -> bool + Sync),
) -> Vec<String> {
	let context = Context {
		repo,
		settings,
		excluded,
	};
	let findings = match linter.0 {
		Kind::Document(linter) => linter.check(&context, path, &repo.docs[path]).expect("the linter runs"),
		Kind::Repository(linter) => linter.check(&context),
	};
	let mut found: Vec<String> = findings
		.iter()
		.map(|f| format!("{}:{}: {}", f.path, f.line, f.message))
		.collect();
	found.sort();
	found
}

fn found(linter: Linter, repo: &Repo<'_>, path: &str) -> Vec<String> {
	found_excluding(linter, repo, path, &Settings::default(), &|_| false)
}

fn found_with(linter: Linter, repo: &Repo<'_>, path: &str, settings: &Settings) -> Vec<String> {
	found_excluding(linter, repo, path, settings, &|_| false)
}

#[test]
fn registry() {
	let names = super::names();
	let mut unique = names.clone();
	unique.sort_unstable();
	unique.dedup();
	assert_eq!(unique.len(), names.len(), "{names:?}");
	for about in ALL.iter().map(|l| l.about()) {
		assert!(!about.description.is_empty(), "{}", about.name);
		let parts: Vec<&str> = about.since.strip_prefix('v').unwrap_or_default().split('.').collect();
		let release = parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok());
		assert!(release, "{}: since {}", about.name, about.since);
	}
	let repository: Vec<&str> = ALL
		.iter()
		.filter(|l| l.is_repository())
		.map(|l| l.about().name)
		.collect();
	assert_eq!(repository, ["index", "readme-links"]);
}

#[test]
fn broken_links() {
	let f = files(&[
		(
			"README.md",
			"# Top\n[ok](docs/a.md#x) [dir](docs) [self](#nope)\n[gone](missing.md) [img](logo.png#x)\n\
			 `[c](missing.md)` [web](https://x.y) [up](../other.md) [deep](docs/../../x.md)\n",
		),
		("docs/a.md", "# A\n"),
	]);
	assert_eq!(
		found(Linter::document(&links::Links), &repo(&f), "README.md"),
		[
			"README.md:3: broken link: logo.png#x",
			"README.md:3: broken link: missing.md",
			"README.md:4: link leaves the repository: ../other.md",
			"README.md:4: link leaves the repository: docs/../../x.md",
		]
	);
}

#[test]
fn broken_anchors() {
	let f = files(&[
		(
			"README.md",
			"# Top\n[a](docs/a.md#the-lang-cookie) [b](docs/a.md#nope) [c](#top) [d](#nope)\n\
			 [e](missing.md#x) [f](../x.md#y) [g](img.png#z)\n",
		),
		("docs/a.md", "# The `lang` cookie\n[up](../README.md#TOP)\n"),
		("img.png", ""),
	]);
	let repo = repo(&f);
	let anchors = Linter::document(&anchors::Anchors);
	assert_eq!(
		found(anchors, &repo, "README.md"),
		[
			"README.md:2: broken anchor: #nope",
			"README.md:2: broken anchor: docs/a.md#nope"
		]
	);
	assert_eq!(found(anchors, &repo, "docs/a.md"), [] as [&str; 0]);
}

#[test]
fn alerts() {
	let f = files(&[(
		"a.md",
		"> [!NOTE]\n>\n> fine\n\n> [!INFO]\n>\n> x\n\n> [!TIP]\n> no gap\n\n```\n> [!BAD]\n```",
	)]);
	assert_eq!(
		found(Linter::document(&alerts::Alerts), &repo(&f), "a.md"),
		[
			"a.md:5: unknown alert type [!INFO] (allowed: NOTE, TIP, IMPORTANT, WARNING, CAUTION)",
			"a.md:9: alert tag must be followed by an empty > line",
		]
	);
}

#[test]
fn blockquotes() {
	let f = files(&[("a.md", "> one\n> two\n\n> three\nlazy\n")]);
	let found = found(Linter::document(&blockquotes::Blockquotes), &repo(&f), "a.md");
	assert_eq!(found, ["a.md:5: blockquote continues on a line without >"]);
}

#[test]
fn fences() {
	let f = files(&[("a.md", "x\n```go\ncode"), ("b.md", "```\ncode\n```")]);
	let repo = repo(&f);
	let fences = Linter::document(&fences::Fences);
	assert_eq!(found(fences, &repo, "a.md"), ["a.md:2: code block is never closed"]);
	assert_eq!(found(fences, &repo, "b.md"), [] as [&str; 0]);
}

#[test]
fn line_length() {
	let long = "word ".repeat(30);
	let content = [
		long.clone(),
		format!("| {long} |"),
		"```".to_owned(),
		long.clone(),
		"```".to_owned(),
		format!("[![badge](https://img.shields.io/x)](https://example.com/{long})"),
		format!("see https://example.com/{}", "x".repeat(80)),
		"ż".repeat(120),
	]
	.join("\n");
	let f = files(&[("a.md", &content)]);
	let repo = repo(&f);
	let line_length = Linter::document(&line_length::LineLength);
	assert_eq!(
		found(line_length, &repo, "a.md"),
		["a.md:1: line is 150 characters, limit 120"]
	);
	let wide = Settings {
		line_length: LineLength {
			max: NonZeroUsize::new(200).expect("not zero"),
		},
		..Settings::default()
	};
	assert_eq!(found_with(line_length, &repo, "a.md", &wide), [] as [&str; 0]);
}

#[test]
fn index() {
	let f = files(&[
		("CONTENTS.md", "[r](README.md)\n[a](docs/a.md#x)\n[self](#top)\n"),
		("README.md", "x"),
		("docs/a.md", "x"),
		("docs/b.md", "x"),
		("other/IDX.md", "x"),
	]);
	let repo = repo(&f);
	let index = Linter::repository(&index::Index);
	assert_eq!(
		found(index, &repo, ""),
		[
			"CONTENTS.md:1: document not listed: docs/b.md",
			"CONTENTS.md:1: document not listed: other/IDX.md"
		]
	);
	let excluded = found_excluding(index, &repo, "", &Settings::default(), &|p| p == "other/IDX.md");
	assert_eq!(excluded, ["CONTENTS.md:1: document not listed: docs/b.md"]);
	let missing = Settings {
		index: Index {
			file: "MISSING.md".to_owned(),
		},
		..Settings::default()
	};
	assert_eq!(found_with(index, &repo, "", &missing), [] as [&str; 0]);
}

#[test]
fn readme_links() {
	let f = files(&[
		("README.md", "[c](./CONTENTS.md)\n"),
		("CONTENTS.md", "x"),
		("GLOSSARY.md", "x"),
	]);
	let found = found(Linter::repository(&readme_links::ReadmeLinks), &repo(&f), "");
	assert_eq!(found, ["README.md:1: does not link GLOSSARY.md"]);
}

#[test]
fn reference_links() {
	let f = files(&[
		(
			"README.md",
			"# Top\n\n[ok][guide] [gone][missing] [bad][part] `[c][code]`\n\n\
			 [guide]: docs/guide.md#intro\n[part]: docs/guide.md#nope\n[dead]: nowhere.md\n",
		),
		("docs/guide.md", "Intro\n=====\n"),
	]);
	let repo = repo(&f);
	assert_eq!(
		found(Linter::document(&links::Links), &repo, "README.md"),
		[
			"README.md:3: undefined reference: [missing]",
			"README.md:7: broken link: nowhere.md"
		]
	);
	assert_eq!(
		found(Linter::document(&anchors::Anchors), &repo, "README.md"),
		["README.md:6: broken anchor: docs/guide.md#nope"]
	);
}

#[test]
fn anchors_of_setext_headings_and_html_ids() {
	let f = files(&[
		(
			"README.md",
			"[a](docs/a.md#setext-title) [b](docs/a.md#custom) [c](docs/a.md#gone)\n",
		),
		("docs/a.md", "Setext title\n------------\n\n<a id=\"custom\"></a>\n"),
	]);
	let found = found(Linter::document(&anchors::Anchors), &repo(&f), "README.md");
	assert_eq!(found, ["README.md:1: broken anchor: docs/a.md#gone"]);
}

#[test]
fn link_forms() {
	let f = files(&[
		(
			"README.md",
			"# Top\n\nA [split\nlink](docs/my%20file.md#a-heading) and [angle](<docs/my file.md>) and\n\
			 [query](docs/my%20file.md?plain=1) and [nested [text]](docs/b(1).md) and [gone\ntoo](docs/none.md).\n\n\
			 [^1]: a footnote is not a link\n",
		),
		("docs/my file.md", "# A heading\n"),
		("docs/b(1).md", "# B\n"),
	]);
	let repo = repo(&f);
	assert_eq!(
		found(Linter::document(&links::Links), &repo, "README.md"),
		["README.md:6: broken link: docs/none.md"]
	);
	assert_eq!(
		found(Linter::document(&anchors::Anchors), &repo, "README.md"),
		[] as [&str; 0]
	);
}

#[test]
fn index_sees_split_links() {
	let f = files(&[
		("CONTENTS.md", "[The\nreadme](README.md) and [b](docs/b%20c.md)\n"),
		("README.md", "x"),
		("docs/b c.md", "x"),
	]);
	assert_eq!(found(Linter::repository(&index::Index), &repo(&f), ""), [] as [&str; 0]);
}

#[test]
fn fence_language() {
	let f = files(&[("a.md", "```go\nx\n```\n\n```\ny\n```\n\n~~~ text\nz\n~~~\n")]);
	let found = found(Linter::document(&fence_language::FenceLanguage), &repo(&f), "a.md");
	assert_eq!(found, ["a.md:5: code block without a language"]);
}

#[test]
fn heading_increment() {
	let f = files(&[("a.md", "# A\n\n## B\n\n#### D\n\n## E\n\nSetext\n------\n\n### F\n")]);
	let found = found(
		Linter::document(&heading_increment::HeadingIncrement),
		&repo(&f),
		"a.md",
	);
	assert_eq!(found, ["a.md:5: heading level jumps from 2 to 4"]);
}

#[test]
fn image_alt() {
	let f = files(&[("a.md", "![logo](a.png) ![](b.png) [![](badge.svg)](https://x.y)\n")]);
	assert_eq!(
		found(Linter::document(&image_alt::ImageAlt), &repo(&f), "a.md"),
		[
			"a.md:1: image without alternative text: b.png",
			"a.md:1: image without alternative text: badge.svg"
		]
	);
}

#[test]
fn empty_links() {
	let f = files(&[
		(
			"README.md",
			"[no target]() [](README.md) [![badge](a.png)](README.md) [fine](README.md)\n",
		),
		("a.png", ""),
	]);
	assert_eq!(
		found(Linter::document(&links::Links), &repo(&f), "README.md"),
		[
			"README.md:1: link without a target: [no target]()",
			"README.md:1: link without text: [](README.md)"
		]
	);
}

#[test]
fn table_columns() {
	let f = files(&[("a.md", "| a | b |\n|---|---|\n| 1 | 2 |\n| 1 | 2 | 3 |\n| 1 |\n")]);
	let found = found(Linter::document(&table_columns::TableColumns), &repo(&f), "a.md");
	assert_eq!(
		found,
		["a.md:4: row has 3 cells, the header 2; the extra cells are not shown"]
	);
}

#[test]
fn unused_definitions() {
	let f = files(&[(
		"a.md",
		"[x][full] [collapsed][] [shortcut] ![img][pic]\n\n\
		 [full]: a.md\n[collapsed]: b.md\n[shortcut]: c.md\n[pic]: d.png\n[unused]: e.md\n",
	)]);
	let found = found(
		Linter::document(&unused_definitions::UnusedDefinitions),
		&repo(&f),
		"a.md",
	);
	assert_eq!(found, ["a.md:7: unused definition: [unused]"]);
}

/// Checks that `linter` reports, in each document alone in a repository, the lines its case lists.
fn check_lines(linter: Linter, cases: &[(&str, &[usize])]) {
	for (text, want) in cases {
		let f = files(&[("a.md", text)]);
		let mut lines: Vec<usize> = found(linter, &repo(&f), "a.md")
			.iter()
			.map(|issue| {
				issue
					.split(':')
					.nth(1)
					.and_then(|line| line.parse().ok())
					.expect("an issue has a line")
			})
			.collect();
		lines.sort_unstable();
		assert_eq!(lines, *want, "{} on {text:?}", linter.about().name);
	}
}

#[test]
fn duplicate_definitions() {
	check_lines(
		Linter::document(&duplicate_definitions::DuplicateDefinitions),
		&[
			("[a]: x\n[a]: y\n", &[2]),
			("[a]: x\n[A]: y\n", &[2]),
			("[a]: x\n[b]: y\n", &[]),
			("[a]: x\n\n> [a]: y\n", &[3]),
			("[a]: x\n\n- [a]: y\n", &[3]),
			("[a]: x\n\n```\n[a]: y\n```\n", &[]),
			("[a]: x\n\ntext\n[a]: y\n", &[]),
			("[a]: x\n\n    [a]: y\n", &[]),
		],
	);
	let f = files(&[("a.md", "[a]: x\n[a]: y\n")]);
	let found = found(
		Linter::document(&duplicate_definitions::DuplicateDefinitions),
		&repo(&f),
		"a.md",
	);
	assert_eq!(found, ["a.md:2: [a] is already defined on line 1"]);
}

#[test]
fn heading_punctuation() {
	check_lines(
		Linter::document(&heading_punctuation::HeadingPunctuation),
		&[
			("# A.\n", &[1]),
			("# A?\n", &[]),
			("# A\n\n## B:\n", &[3]),
			("A!\n===\n", &[1]),
			("# Q：\n", &[1]),
			("# Café\n", &[]),
			("```\n# x.\n```\n", &[]),
		],
	);
}

#[test]
fn list_indent() {
	check_lines(
		Linter::document(&list_indent::ListIndent),
		&[
			("- a\n - b\n", &[2]),
			("- a\n- b\n", &[]),
			(" 9. a\n10. b\n", &[]),
			("1. a\n10. b\n", &[]),
			("- a\n\n  - b\n  - c\n", &[]),
			("- a\n  - b\n   - c\n", &[3]),
			("> - a\n>  - b\n", &[2]),
		],
	);
}

#[test]
fn reversed_links() {
	check_lines(
		Linter::document(&reversed_links::ReversedLinks),
		&[
			("(text)[a.md]\n", &[1]),
			("(text)[ref]\n\n[ref]: a.md\n", &[]),
			("`(text)[a.md]`\n", &[]),
			("[text](a.md)\n", &[]),
		],
	);
}

#[test]
fn link_text() {
	check_lines(
		Linter::document(&link_text::LinkText),
		&[
			("[here](a.md)\n", &[1]),
			("[Click here!](a.md)\n", &[1]),
			("[the guide](a.md)\n", &[]),
		],
	);
}

#[test]
fn blank_line_in_blockquote() {
	check_lines(
		Linter::document(&blank_line_in_blockquote::BlankLineInBlockquote),
		&[
			("> a\n\n> b\n", &[2]),
			("> a\n>\n> b\n", &[]),
			("> [!NOTE]\n> a\n\n> [!TIP]\n> b\n", &[]),
			("> a\n\ntext\n\n> b\n", &[]),
		],
	);
}

#[test]
fn code_block_style() {
	check_lines(
		Linter::document(&code_block_style::CodeBlockStyle),
		&[
			("text\n\n    code\n", &[3]),
			("```\ncode\n```\n", &[]),
			("- a\n\n      code\n", &[3]),
		],
	);
}

#[test]
fn first_heading() {
	check_lines(
		Linter::document(&first_heading::FirstHeading),
		&[
			("# A\n", &[]),
			("## A\n", &[1]),
			("---\nt: x\n---\n# A\n", &[]),
			("<p>badge</p>\n\n# A\n", &[]),
			("text\n\n# A\n", &[1]),
			("", &[]),
		],
	);
}

#[test]
fn single_title() {
	check_lines(
		Linter::document(&single_title::SingleTitle),
		&[("# A\n\n# B\n", &[3]), ("# A\n\n## B\n", &[])],
	);
}

#[test]
fn heading_without_space() {
	check_lines(
		Linter::document(&heading_without_space::HeadingWithoutSpace),
		&[("#Title\n", &[1]), ("# Title\n", &[]), ("```\n#x\n```\n", &[])],
	);
}

#[test]
fn duplicate_headings() {
	check_lines(
		Linter::document(&duplicate_headings::DuplicateHeadings),
		&[
			("# A\n\n## S\n\n### X\n\n## T\n\n### X\n", &[]),
			("# A\n\n## S\n\n## S\n", &[5]),
			("# A\n\n## s\n\n## S\n", &[5]),
		],
	);
}

#[test]
fn emphasis_as_heading() {
	check_lines(
		Linter::document(&emphasis_as_heading::EmphasisAsHeading),
		&[
			("**Setup**\n", &[1]),
			("**Note:** text\n", &[]),
			("**Done.**\n", &[]),
			("- **x**\n", &[]),
		],
	);
}

#[test]
fn every_example_works() {
	for &linter in ALL {
		let about = linter.about();
		for (document, reported) in [(about.example.bad, true), (about.example.good, false)] {
			let mut f = files(about.example.files);
			f.insert(about.example.path.to_owned(), document.to_owned());
			let found = found(linter, &repo(&f), about.example.path);
			assert_eq!(
				!found.is_empty(),
				reported,
				"{}: {document:?} gives {found:?}",
				about.name
			);
		}
	}
}

/// Answers from its files and counts how often each path is asked for.
struct Counting {
	files: BTreeMap<String, String>,
	asked: Mutex<HashMap<String, usize>>,
}

impl Files for Counting {
	fn read(&self, path: &str) -> io::Result<Vec<u8>> {
		self.files.read(path)
	}

	fn exists(&self, path: &str) -> io::Result<bool> {
		*self
			.asked
			.lock()
			.expect("no test panics while holding it")
			.entry(path.to_owned())
			.or_default() += 1;
		self.files.exists(path)
	}
}

#[test]
fn exists_asks_the_files_only_for_unknown_paths() {
	let counting = Counting {
		files: files(&[("logo.png", "")]),
		asked: Mutex::default(),
	};
	let docs = BTreeMap::from([("docs/guide/a.md".to_owned(), Document::parse("x"))]);
	let repo = Repo::new(&counting, docs);
	let paths = [
		("docs/guide/a.md", true),
		("docs/guide", true),
		("docs", true),
		(".", true),
		("logo.png", true),
		("gone.md", false),
	];
	for _ in 0..3 {
		for (path, want) in paths {
			assert_eq!(repo.exists(path).expect("the files answer"), want, "{path}");
		}
	}
	let asked = counting.asked.into_inner().expect("no test panicked while holding it");
	assert_eq!(
		asked,
		HashMap::from([("logo.png".to_owned(), 1), ("gone.md".to_owned(), 1)])
	);
}
