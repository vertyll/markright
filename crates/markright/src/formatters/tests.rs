use proptest::prelude::*;

use super::settings::{self, Settings};
use super::width::display_width;
use super::{ALL, Fix, Formatter, arrows, blank_lines, final_newline, list_marker, tables, trailing_whitespace};
use crate::markdown::Marker;
use crate::markdown::{Document, table_cells};

/// Runs `formatter` on `text`, parsed, with the lines `in_code` marks as code as well, as a diagram block would be.
fn apply(formatter: &dyn Formatter, text: &str, in_code: &[usize]) -> String {
	let mut d = Document::parse(text);
	for &i in in_code {
		d.lines[i].code = true;
	}
	formatter.apply(&d, &Settings::default()).lines.join("\n")
}

fn fix(formatter: &dyn Formatter, text: &str) -> Fix {
	formatter.apply(&Document::parse(text), &Settings::default())
}

#[test]
fn arrows() {
	let cases = [
		("a -> b", "a → b"),
		("a->b", "a → b"),
		("a <- b", "a ← b"),
		("a <-> b", "a ↔ b"),
		("a --> b", "a → b"),
		("a->->b", "a → → b"),
		("→→→", "→ → →"),
		("a→b", "a → b"),
		("no arrows here", "no arrows here"),
		("run `a -> b` then a -> b", "run `a -> b` then a → b"),
		("`x`->y", "`x` → y"),
		("text <!-- x -> y --> and a -> b", "text <!-- x -> y --> and a → b"),
		("[a->b](docs/a->b.md) and a->b", "[a → b](docs/a->b.md) and a → b"),
	];
	for (input, want) in cases {
		assert_eq!(apply(&arrows::Arrows, input, &[]), want, "{input:?}");
	}
}

#[test]
fn arrows_leave_html_blocks_alone() {
	let input = "<!-- x -> y --> and a -> b\n\ntext a -> b";
	assert_eq!(
		apply(&arrows::Arrows, input, &[]),
		"<!-- x -> y --> and a -> b\n\ntext a → b"
	);
}

#[test]
fn fixes_skip_code() {
	let input = "a -> b  \nA->>B: call  ";
	assert_eq!(apply(&arrows::Arrows, input, &[1]), "a → b  \nA->>B: call  ");
	assert_eq!(
		apply(&trailing_whitespace::TrailingWhitespace, input, &[1]),
		"a -> b\nA->>B: call  "
	);
}

#[test]
fn tables() {
	let input = [
		"text",
		"| Name | Role |",
		"|---|:-:|",
		"| `a|b` | the żółw |",
		"| c |",
		"after",
	]
	.join("\n");
	let want = [
		"text",
		"| Name  |   Role   |",
		"|-------|:--------:|",
		"| `a|b` | the żółw |",
		"| c     |          |",
		"after",
	]
	.join("\n");
	assert_eq!(apply(&tables::Tables, &input, &[]), want);
}

#[test]
fn tables_keep_their_indent_and_are_idempotent() {
	let once = apply(&tables::Tables, "   | a | b |\n   |---|---|\n   | long cell | x |", &[]);
	assert!(once.starts_with("   | a         | b |"), "indent lost:\n{once}");
	assert_eq!(apply(&tables::Tables, &once, &[]), once);
}

#[test]
fn tables_need_a_gfm_delimiter_row() {
	for input in [
		"| a | b |\n| c | d |",
		"|\n|\n| ",
		"| a | b |\n|   |   |\n| c | d |",
		"| a | b |\n|---|\n| c | d |",
	] {
		assert_eq!(apply(&tables::Tables, input, &[]), input);
	}
}

#[test]
fn cells() {
	let cases: [(&str, &[&str]); 4] = [
		(r"| a \| b | c |", &[r"a \| b", "c"]),
		(
			"| fenced by ```` ``` ```` or `~~~` | x |",
			&["fenced by ```` ``` ```` or `~~~`", "x"],
		),
		("| ``a | b`` | c |", &["``a | b``", "c"]),
		("| unmatched ` tick | c |", &["unmatched ` tick", "c"]),
	];
	for (row, want) in cases {
		assert_eq!(table_cells(row), want, "{row:?}");
	}
}

#[test]
fn display_width_counts_columns() {
	let cases = [
		("abc", 3),
		("żółw", 4),
		("→", 1),
		("✅ ok", 5),
		("❌", 2),
		("⚠️", 2),
		("🚀", 2),
		("漢字", 4),
		("é", 1),
		("👩‍💻", 4),
	];
	for (text, want) in cases {
		assert_eq!(display_width(text), want, "{text:?}");
	}
}

#[test]
fn tables_align_emoji_by_display_width() {
	let input = "| A | B |\n|---|---|\n| ✅ yes | x |\n| plain | y |";
	let want = "| A      | B |\n|--------|---|\n| ✅ yes | x |\n| plain  | y |";
	assert_eq!(apply(&tables::Tables, input, &[]), want);
}

#[test]
fn tables_honor_column_alignment() {
	let input = "| L | C | R |\n|:--|:-:|--:|\n| left | mid | 1 |\n| x | y | 100 |";
	let want = "| L    |  C  |   R |\n|:-----|:---:|----:|\n| left | mid |   1 |\n| x    |  y  | 100 |";
	assert_eq!(apply(&tables::Tables, input, &[]), want);
}

#[test]
fn tables_with_extra_cells_are_left_alone() {
	let input = "| a | b |\n|---|---|\n| 1 | 2 | 3 |";
	assert_eq!(apply(&tables::Tables, input, &[]), input);
	assert_eq!(
		apply(&tables::Tables, "| a | b |\n|---|---|\n| 1 |", &[]),
		"| a | b |\n|---|---|\n| 1 |   |"
	);
}

#[test]
fn tables_follow_the_parser() {
	let cases = [
		// A table without body rows ends at its delimiter row, which still takes the header's widths.
		(
			"| Field | Holds |\n|-------------|---|\n\n| `status` | the HTTP status |",
			"| Field | Holds |\n|-------|-------|\n\n| `status` | the HTTP status |",
		),
		// A line of text right after a table is a row in GFM; it stays as written, the rows above it are padded.
		(
			"| a | b |\n|---|---|\n| long | x |\nafter",
			"| a    | b |\n|------|---|\n| long | x |\nafter",
		),
		// A table in a blockquote is left alone.
		(
			"> | a | b |\n> |---|---|\n> | long | x |",
			"> | a | b |\n> |---|---|\n> | long | x |",
		),
	];
	for (input, want) in cases {
		assert_eq!(apply(&tables::Tables, input, &[]), want, "{input:?}");
	}
}

#[test]
fn blank_lines() {
	let fix = fix(&blank_lines::BlankLines, "a\n\n\n\nb\n```\n\n\n```\nc\n\n\n");
	assert_eq!(fix.lines.join("\n"), "a\n\nb\n```\n\n\n```\nc\n\n\n");
	assert_eq!(fix.changed, [3, 4]);
}

#[test]
fn final_newline() {
	let cases: [(&str, &str, &[usize]); 5] = [
		("a\n", "a\n", &[]),
		("a", "a\n", &[1]),
		("a\n\n\n", "a\n", &[2]),
		("a\n  \n", "a\n", &[2]),
		("", "", &[]),
	];
	for (input, want, changed) in cases {
		let fix = fix(&final_newline::FinalNewline, input);
		assert_eq!(
			(fix.lines.join("\n").as_str(), fix.changed.as_slice()),
			(want, changed),
			"{input:?}"
		);
	}
}

#[test]
fn list_marker() {
	let input = "* a\n* b\n\ntext\n\n> * quoted\n\n1. x\n   * nested\n\n- keep\n\n* apart\n\n```\n* code\n```";
	let want = "- a\n- b\n\ntext\n\n> - quoted\n\n1. x\n   - nested\n\n- keep\n\n* apart\n\n```\n* code\n```";
	assert_eq!(apply(&list_marker::ListMarker, input, &[14, 15, 16]), want);
}

#[test]
fn list_marker_keeps_adjacent_lists_apart() {
	let input = "* one list\n+ another list, by its marker";
	assert_eq!(apply(&list_marker::ListMarker, input, &[]), input);
}

#[test]
fn list_marker_follows_its_settings() {
	let d = Document::parse("- a\n+ b\n\ntext\n\n- c\n- d\n\n```\n- code\n```");
	let settings = Settings {
		list_marker: settings::ListMarker { marker: Marker::Star },
	};
	let got = list_marker::ListMarker.apply(&d, &settings).lines.join("\n");
	assert_eq!(got, "- a\n+ b\n\ntext\n\n* c\n* d\n\n```\n- code\n```");
}

#[test]
fn list_marker_settings_accept_only_list_markers() {
	for marker in ["-", "*", "+"] {
		let read: Result<settings::ListMarker, _> = serde_saphyr::from_str(&format!("marker: \"{marker}\""));
		assert!(read.is_ok(), "{marker:?}");
	}
	for marker in ["", "x", "--", "1."] {
		let read: Result<settings::ListMarker, _> = serde_saphyr::from_str(&format!("marker: \"{marker}\""));
		assert!(read.is_err(), "{marker:?} was accepted");
	}
}

#[test]
fn registry() {
	let names = super::names();
	assert_eq!(
		names,
		[
			"trailing-whitespace",
			"arrows",
			"list-marker",
			"tables",
			"blank-lines",
			"final-newline"
		]
	);
	for about in ALL.iter().map(|f| f.about()) {
		assert!(
			!about.description.is_empty() && !about.message.is_empty(),
			"{}",
			about.name
		);
		let parts: Vec<&str> = about.since.strip_prefix('v').unwrap_or_default().split('.').collect();
		assert!(
			parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok()),
			"{}: since {}",
			about.name,
			about.since
		);
	}
}

#[test]
fn every_example_is_fixed_once() {
	for f in ALL {
		let about = f.about();
		let fixed = fix(*f, about.example);
		assert!(!fixed.changed.is_empty(), "{}: the example is not changed", about.name);
		let again = fix(*f, &fixed.lines.join("\n"));
		assert!(
			again.changed.is_empty(),
			"{}: the fixed example is changed again: {:?}",
			about.name,
			again.changed
		);
	}
}

/// Pieces of Markdown that stress the formatters: arrows, tables, code, lists, blank lines and the end of the file.
const PIECES: &[&str] = &[
	"a -> b",
	"| a | b |\n|---|:-:|\n| `x|y` | ż |",
	"x  \n```\ny -> z\n```",
	"<-->",
	"[a](b->c) d->e",
	"| a |\n|---|\n| ✅ ok | 漢字 |",
	"``a`` ` b",
	"\t| a |\n\t|-|",
	"a\n\n\n\nb\n\n\n",
	"```\n\n\n```",
	"x",
	"* a\n* b",
	"> * q\n+ r",
	"\n",
	"<!-- a -> b -->",
	"    code -> x",
	"- [ ] task -> done",
];

proptest! {
	/// Every formatter changes only lines that exist, and leaves what it made as it is.
	#[test]
	fn formatters_are_idempotent(pieces in prop::collection::vec(prop::sample::select(PIECES), 1..8)) {
		let text = pieces.join("\n");
		let d = Document::parse(&text);
		for f in ALL {
			let name = f.about().name;
			let once = f.apply(&d, &Settings::default());
			prop_assert!(once.changed.iter().all(|&n| n >= 1 && n <= d.lines.len()), "{name}: {once:?}");
			let twice = f.apply(&Document::parse(&once.lines.join("\n")), &Settings::default());
			prop_assert!(twice.changed.is_empty() && twice.lines == once.lines, "{name} on {text:?}: {once:?}");
		}
	}
}
