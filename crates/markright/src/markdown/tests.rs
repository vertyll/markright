use std::collections::HashSet;

use proptest::prelude::*;

use super::anchors::slugify;
use super::{Document, LinkKind, Marker, Reference, Table, is_external};

fn code(d: &Document) -> Vec<bool> {
	d.lines.iter().map(|line| line.code).collect()
}

fn anchors(d: &Document) -> Vec<&str> {
	let mut out: Vec<&str> = d.anchors.iter().map(String::as_str).collect();
	out.sort_unstable();
	out
}

#[test]
fn slugs() {
	let cases = [
		("Where do I look?", "where-do-i-look"),
		("The `lang` cookie", "the-lang-cookie"),
		("Zażółć gęślą", "zażółć-gęślą"),
		("RFC 9457: problem details", "rfc-9457-problem-details"),
		("A → B", "a--b"),
		("snake_case and kebab-case", "snake_case-and-kebab-case"),
		("See [the guide](docs/guide.md)", "see-the-guide"),
		("See [the guide][ref]", "see-the-guideref"),
		("The <code>lang</code> cookie", "the-lang-cookie"),
		("**Bold** and _it_ and ~~gone~~", "bold-and-it-and-gone"),
		("![logo](a.png) Title", "-title"),
		(
			"4. Rules (same as [`database-access.md`](database-access.md))",
			"4-rules-same-as-database-accessmd",
		),
		("`<b>` and `_x_`", "b-and-_x_"),
		("Escaped \\* star", "escaped--star"),
		("Fish &amp; chips", "fish--chips"),
	];
	for (heading, want) in cases {
		let d = Document::parse(&format!("# {heading}\n"));
		assert_eq!(anchors(&d), [want], "anchors of {heading:?}");
	}
	assert_eq!(slugify("A_b-C d"), "a_b-c-d");
}

#[test]
fn repeated_headings_get_suffixes() {
	let d = Document::parse("# A\n## Code\n## Code\n## Code\n");
	assert_eq!(anchors(&d), ["a", "code", "code-1", "code-2"]);
}

#[test]
fn fences() {
	let d = Document::parse("```\n# not a heading\n~~~\n```\n# Real\n````\nopen");
	assert_eq!(anchors(&d), ["real"]);
	assert_eq!(code(&d), [true, true, true, true, false, true, true]);
	assert_eq!(d.unclosed_fence, Some(6));
	assert_eq!(d.fences.iter().map(|f| f.line).collect::<Vec<_>>(), [1, 6]);
}

#[test]
fn setext_headings() {
	let d = Document::parse("Title\n=====\n\nSection two\n---\n\n---\n\n- item\n---\n");
	assert_eq!(
		anchors(&d),
		["section-two", "title"],
		"a rule and a list item are not headings"
	);
}

#[test]
fn setext_and_atx_share_duplicate_counting() {
	let d = Document::parse("# Code\n\nCode\n----\n");
	assert_eq!(anchors(&d), ["code", "code-1"]);
}

#[test]
fn html_anchors() {
	let d = Document::parse(
		"<a id=\"Top\"></a>\n<a name='legacy'></a>\n<div id=\"box\">x</div>\n\nText `<a id=\"code\">` and <a id=\"inline\"></a>.\n",
	);
	assert_eq!(
		anchors(&d),
		["box", "inline", "legacy", "top"],
		"an id inside inline code is not an anchor"
	);
}

#[test]
fn indented_code() {
	let d = Document::parse("text\n\n    code -> x\n\n    more\n\nafter\n    not code, continues the paragraph\n");
	assert_eq!(code(&d), [false, false, true, true, true, false, false, false, false]);
}

#[test]
fn indented_text_in_lists_is_not_code() {
	let d = Document::parse(
		&[
			"1. First",
			"",
			"   Continues the item.",
			"",
			"   - Nested",
			"",
			"     Continues the nested item.",
			"",
			"         code in the nested item",
			"",
			"Paragraph.",
			"",
			"    code again",
		]
		.join("\n"),
	);
	let want = [
		false, false, false, false, false, false, false, false, true, false, false, false, true,
	];
	assert_eq!(code(&d), want);
}

fn links(d: &Document) -> Vec<(&str, usize)> {
	d.links.iter().map(|l| (l.target.as_str(), l.line)).collect()
}

#[test]
fn inline_links_and_definitions() {
	let d = Document::parse(
		&[
			"[a](a.md) ![i](img.png \"title\") `[c](code.md)` [e](https://x.y) [h](#here)",
			"See the [long",
			"text](docs/long.md#part) and [nested [brackets]](n.md) and [parens](file(1).md).",
			"[angle](<my file.md>) [titled](t.md 'single') [paren title](p.md (title))",
			"[empty]() [broken](a b c) [not a link] `code",
			"[span](x.md)` [after](after.md) [escaped](a\\_b.md)",
			"",
			"```",
			"[fenced](fenced.md)",
			"```",
			"[ref]: <def target.md>",
			"[^1]: footnote, not a definition",
		]
		.join("\n"),
	);
	let want = [
		("a.md", 1),
		("img.png", 1),
		("https://x.y", 1),
		("#here", 1),
		("docs/long.md#part", 3),
		("n.md", 3),
		("file(1).md", 3),
		("my file.md", 4),
		("t.md", 4),
		("p.md", 4),
		("", 5),
		("after.md", 6),
		("a_b.md", 6),
		("def target.md", 11),
	];
	assert_eq!(links(&d), want);
}

#[test]
fn reference_links() {
	let d = Document::parse(
		"See [the guide][Guide  Doc] and [Other][] and `[c][code]` and [^1] and [gone][nowhere]\nand [split\ntext][other] \
		 and [missing][].\n\n[guide doc]: docs/guide.md\n[other]: other.md#part\n> [quoted]: q.md\n",
	);
	assert_eq!(
		d.undefined_references,
		[
			Reference {
				label: "nowhere".into(),
				line: 1
			},
			Reference {
				label: "missing".into(),
				line: 3
			}
		]
	);
	assert_eq!(
		d.used_labels,
		HashSet::from(["guide doc".to_owned(), "other".to_owned()])
	);
	let guide = &d.definitions["guide doc"];
	assert_eq!((guide.target.as_str(), guide.line), ("docs/guide.md", 5));
	assert_eq!(d.definitions["quoted"].line, 7, "a definition in a blockquote is one");
}

#[test]
fn external_targets() {
	for (target, want) in [
		("https://x.y", true),
		("mailto:a@b.c", true),
		("a.md", false),
		("#x", false),
	] {
		assert_eq!(is_external(target), want, "{target}");
	}
}

#[test]
#[expect(
	clippy::single_range_in_vec_init,
	reason = "each line holds a list of ranges, here often one"
)]
fn protected_spans() {
	let lines = [
		"[ref]: a->b.md",
		"a `code ->",
		"still code` b -> c [x](y->z.md) [long",
		"text](q->r.md)",
		"",
		"<!-- a comment ->",
		"",
		"over a blank line --> end",
	];
	let d = Document::parse(&lines.join("\n"));
	assert!(
		d.lines[5].code && d.lines[6].code && d.lines[7].code,
		"a comment over a blank line is an HTML block"
	);
	let want: [&[std::ops::Range<usize>]; 8] = [&[7..14], &[3..10], &[0..10, 23..30], &[6..13], &[], &[], &[], &[]];
	for (i, want) in want.iter().enumerate() {
		assert_eq!(d.lines[i].protected, *want, "line {} {:?}", i + 1, lines[i]);
	}
}

#[test]
fn link_destinations_are_placed() {
	let d = Document::parse(
		"See [a](docs/a.md) and ![b](<img b.png> \"t\").\n\n[c]: <docs/c.md>\n[e]: docs/e.md#x\n\nUse [d][c].\n",
	);
	assert_eq!(d.links.len(), 4, "two inline links and two definitions: {:?}", d.links);
	for l in &d.links {
		assert!(!l.destination.is_empty(), "{} has no destination", l.target);
		assert_eq!(&d.lines[l.line - 1].text[l.destination.clone()], l.target);
	}
}

#[test]
fn link_texts() {
	let d = Document::parse(
		"[The *guide*](a.md) ![](b.png) [![badge](c.svg)](d.md) [](e.md) [`code`](f.md)

[x]: g.md
",
	);
	let kinds: Vec<LinkKind> = d.links.into_iter().map(|l| l.kind).collect();
	let inline = |text: &str, has_content| LinkKind::Inline {
		text: text.to_owned(),
		has_content,
	};
	let image = |alt: &str| LinkKind::Image { alt: alt.to_owned() };
	assert_eq!(
		kinds,
		[
			inline("The guide", true),
			image(""),
			inline("", true),
			image("badge"),
			inline("", false),
			inline("code", true),
			LinkKind::Definition,
		]
	);
}

#[test]
fn bullets_and_adjacent_lists() {
	let d = Document::parse("* a\n+ b\n\n---\n\n- c\n\n1. x\n   * y\n");
	let got: Vec<(usize, usize, Marker, bool)> = d
		.bullets
		.iter()
		.map(|b| (b.line, b.column, b.marker, b.adjacent))
		.collect();
	assert_eq!(
		got,
		[
			(1, 0, Marker::Star, true),
			(2, 0, Marker::Plus, true),
			(6, 0, Marker::Dash, true),
			(9, 3, Marker::Star, false)
		]
	);
	let d = Document::parse("+ a\n\n[x]: y.md\n+ b\n");
	assert!(
		d.bullets.iter().all(|b| !b.adjacent),
		"a definition keeps two lists apart: {:?}",
		d.bullets
	);
}

#[test]
fn tables_and_lazy_lines() {
	let d = Document::parse("| a | b |\n|---|---|\n| 1 | 2 |\ntext\n\n> one\nlazy\n> | x |\n> |---|\n");
	assert_eq!(
		d.tables,
		[
			Table {
				header: 1,
				body: vec![3, 4]
			},
			Table {
				header: 8,
				body: vec![]
			}
		]
	);
	assert_eq!(d.lazy_lines, [7]);
}

#[test]
fn headings_have_levels_and_lines() {
	let d = Document::parse("# A\n\nB\n---\n\n### C\n");
	let got: Vec<(u8, usize)> = d.headings.iter().map(|h| (h.level, h.line)).collect();
	assert_eq!(got, [(1, 1), (2, 3), (3, 6)]);
}

#[test]
fn display_gives_the_content_back() {
	for content in ["", "a", "a\n", "a\r\nb\n\n", "# x\n```\ny"] {
		assert_eq!(Document::parse(content).to_string(), content);
	}
}

/// Pieces of Markdown that stress the parser, which documents are built from.
const PIECES: &[&str] = &[
	"# A\n",
	"text\n",
	"```\ncode\n",
	"Title\n===\n",
	"- a\n\n      code\n",
	"[x][y]\n",
	"[y]: z.md\n",
	"<a id=\"x\">",
	"\t\tx",
	"[a\nb](c.md)",
	"[a [b]](c(1).md \"t\")",
	"[x](<a b.md>)",
	"> [!NOTE]\n>\n> y",
	"| a |\n|---|\n",
	"`a\nb`",
	"> q\nlazy\n",
	"* a\n+ b\n",
	"![](<>)",
	"[]()",
	"\\",
	"<!--",
	"~~~",
	"\n\n",
	"[x]:\n  <y z.md>\n",
	"é ✅ 漢",
	"&amp;",
	"[a](b \"c\nd\")",
];

proptest! {
	/// Every parsed document holds its content, and every position it keeps is inside it.
	#[test]
	fn positions_stay_inside_the_document(
		pieces in prop::collection::vec((prop::sample::select(PIECES), prop::sample::select(&["", "\n", "x", "\n\n"][..])), 1..10)
	) {
		let content: String = pieces.iter().flat_map(|(piece, separator)| [*piece, *separator]).collect();
		check_positions(&content);
	}
}

/// Checks what every parsed document must hold: one code flag per line, the content back, and every position inside
/// the document.
fn check_positions(content: &str) {
	let d = Document::parse(content);
	assert_eq!(d.to_string(), content);
	assert!(d.unclosed_fence.is_none_or(|line| line >= 1 && line <= d.lines.len()));
	for (line, span) in d
		.lines
		.iter()
		.flat_map(|line| line.protected.iter().map(move |s| (&line.text, s)))
	{
		assert!(
			span.end <= line.len() && span.start <= span.end,
			"{span:?} outside {line:?}"
		);
		assert!(line.is_char_boundary(span.start) && line.is_char_boundary(span.end));
	}
	for l in &d.links {
		assert!(l.line >= 1 && l.line <= d.lines.len(), "{l:?} in {content:?}");
		assert!(
			l.destination.end <= d.lines[l.line - 1].text.len(),
			"{l:?} in {content:?}"
		);
	}
}
