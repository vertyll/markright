use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::{Mode, Report, Runner};
use crate::Error;
use crate::config::{Config, Syntax, YAML_FILE};

fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
	let root = tempfile::tempdir().unwrap();
	for (name, content) in files {
		let path = root.path().join(name);
		fs::create_dir_all(path.parent().unwrap()).unwrap();
		fs::write(path, content).unwrap();
	}
	root
}

fn config(text: &str) -> Config {
	Config::parse(text, YAML_FILE, Syntax::Yaml).unwrap()
}

fn run(root: &Path, config: &Config, mode: Mode) -> Report {
	Runner { root, config, mode }.run(&AtomicBool::new(false)).unwrap()
}

/// What a run prints: the formatted documents, then the issues.
fn output(report: &Report) -> String {
	let formatted = report.formatted.iter().map(|p| format!("formatted {p}\n"));
	formatted
		.chain(report.issues.iter().map(|i| format!("{i}\n")))
		.collect()
}

fn read(root: &Path, name: &str) -> String {
	fs::read_to_string(root.join(name)).unwrap()
}

const CLEAN: &[(&str, &str)] = &[
	("README.md", "# Project\n\n[Contents](CONTENTS.md)\n"),
	(
		"CONTENTS.md",
		"# Contents\n\n| Document | Covers |\n|---|---|\n| [README](README.md) | start |\n| [A](docs/a.md) | a → b |\n",
	),
	(
		"docs/a.md",
		"# A\n\n> [!NOTE]\n>\n> see [top](../README.md#project)\n\n```mermaid\nA->>B: call\n```\n",
	),
];

#[test]
fn check_reports_formatting_and_lint_without_writing() {
	let root = tree(&[("a.md", "# A\n\na -> b  \n[x](gone.md)\n")]);
	let report = run(root.path(), &Config::default(), Mode::Lint);
	assert_eq!(
		output(&report),
		"a.md:3: ASCII arrow or missing space around an arrow (arrows)
\
		 a.md:3: trailing whitespace (trailing-whitespace)
\
		 a.md:4: broken link: gone.md (links)
"
	);
	assert_eq!(read(root.path(), "a.md"), "# A\n\na -> b  \n[x](gone.md)\n");
}

#[test]
fn fmt_writes_and_reports_no_lint() {
	let root = tree(CLEAN);
	fs::write(root.path().join("docs/b.md"), "# B\n\na->b [x](gone.md)\n").unwrap();
	let out = output(&run(root.path(), &Config::default(), Mode::Format));
	assert_eq!(
		out,
		"formatted CONTENTS.md
formatted docs/b.md
",
		"fmt formats and reports no lint"
	);
	assert_eq!(read(root.path(), "docs/b.md"), "# B\n\na → b [x](gone.md)\n");
	assert!(read(root.path(), "CONTENTS.md").contains(
		"
|---------------------|--------|
"
	));
}

#[test]
fn fix_writes_and_reports_what_is_left() {
	let root = tree(&[("a.md", "# A\n\na -> b  \n[x](gone.md)\n")]);
	let report = run(root.path(), &Config::default(), Mode::Fix);
	assert_eq!(
		output(&report),
		"formatted a.md\na.md:4: broken link: gone.md (links)\n",
		"the formatters' fixes are written, and only the lint is reported"
	);
	assert_eq!(read(root.path(), "a.md"), "# A\n\na → b\n[x](gone.md)\n");
	assert_eq!(
		output(&run(root.path(), &Config::default(), Mode::Fix)),
		"a.md:4: broken link: gone.md (links)\n"
	);
}

#[test]
fn check_is_clean_after_fmt() {
	let root = tree(CLEAN);
	run(root.path(), &Config::default(), Mode::Format);
	assert_eq!(output(&run(root.path(), &Config::default(), Mode::Lint)), "");
}

#[test]
fn disabled_linters_and_formatters() {
	let root = tree(&[("a.md", &format!("# A\n\na -> b\n[x](gone.md)\n{}\n", "x".repeat(160)))]);
	let mut config = Config::default();
	config.linters.disable = vec!["links".into(), "line-length".into()];
	config.formatters.disable = vec!["arrows".into()];
	assert_eq!(output(&run(root.path(), &config, Mode::Lint)), "");
}

#[test]
fn exclusions() {
	let long = format!("# Legacy\n\n{}\n[y](gone.md)\n", "x".repeat(160));
	let root = tree(&[
		("CONTENTS.md", "# Contents\n\n[a](a.md)\n"),
		("a.md", "# A\n"),
		("generated/x.md", "a -> b\n[x](gone.md)\n"),
		("docs/legacy.md", &long),
		("docs/legacy-table.md", "| a | b |\n|---|---|\n| long cell | x |\n"),
	]);
	let config = config(
		"version: \"1\"\nlinters:\n  exclusions:\n    paths: ['^generated/', '^docs/legacy-table\\.md$']\n    rules:\n      \
		 - path: '^docs/legacy\\.md$'\n        linters: [line-length]\nformatters:\n  exclusions:\n    \
		 paths: ['^generated/', '^docs/legacy-table\\.md$']\n",
	);
	let out = output(&run(root.path(), &config, Mode::Lint));
	assert_eq!(
		out,
		"CONTENTS.md:1: document not listed: docs/legacy.md (index)
docs/legacy.md:4: broken link: gone.md (links)
"
	);
	run(root.path(), &config, Mode::Format);
	assert_eq!(
		read(root.path(), "generated/x.md"),
		"a -> b\n[x](gone.md)\n",
		"fmt touched an excluded file"
	);
}

#[test]
fn configured_index_and_readme() {
	let root = tree(&[
		("index.md", "# Index\n\n[r](readme.md)\n"),
		("readme.md", "# Readme\n"),
		("GLOSSARY.md", "# Glossary\n"),
	]);
	let config = config(
		"version: \"1\"\nlinters:\n  settings:\n    index:\n      file: index.md\n    readme-links:\n      file: readme.md\n      \
		 links: [index.md, GLOSSARY.md]\n",
	);
	assert_eq!(
		output(&run(root.path(), &config, Mode::Lint)),
		"index.md:1: document not listed: GLOSSARY.md (index)
\
		 readme.md:1: does not link GLOSSARY.md (readme-links)
\
		 readme.md:1: does not link index.md (readme-links)
"
	);
}

#[test]
fn unused_exclusions_are_counted_per_run() {
	let root = tree(&[("used/a.md", "# A\n"), ("b.md", "[x](gone.md)\n")]);
	let config = config(
		"version: \"1\"\nlinters:\n  exclusions:\n    warn-unused: true\n    paths: ['^used/', '^nothing/']\n    rules:\n      \
		 - {path: '^b\\.md$', linters: [links]}\n      - {path: '^c\\.md$', linters: [links]}\n",
	);
	let runner = Runner {
		root: root.path(),
		config: &config,
		mode: Mode::Lint,
	};
	for _ in 0..2 {
		let unused = runner.run(&AtomicBool::new(false)).unwrap().unused_exclusions;
		assert_eq!(
			unused,
			[
				"linters.exclusions.paths[1]: \"^nothing/\" matches no document",
				"linters.exclusions.rules[1] matches no issue"
			]
		);
	}
}

#[test]
fn line_changing_formatters_report_the_original_lines() {
	let root = tree(&[("a.md", "# A\n\n\n\ntext a -> b\n\n\n\nend")]);
	assert_eq!(
		output(&run(root.path(), &Config::default(), Mode::Lint)),
		"a.md:3: more than one blank line in a row (blank-lines)
\
		 a.md:4: more than one blank line in a row (blank-lines)
\
		 a.md:5: ASCII arrow or missing space around an arrow (arrows)
\
		 a.md:7: more than one blank line in a row (blank-lines)
\
		 a.md:8: more than one blank line in a row (blank-lines)
\
		 a.md:9: the file does not end with exactly one newline (final-newline)
"
	);
	run(root.path(), &Config::default(), Mode::Format);
	assert_eq!(read(root.path(), "a.md"), "# A\n\ntext a → b\n\nend\n");
	assert_eq!(output(&run(root.path(), &Config::default(), Mode::Lint)), "");
}

#[test]
fn formatter_settings_reach_the_formatter() {
	let root = tree(&[("a.md", "# A\n\n- one\n- two\n")]);
	let config = config("version: \"1\"\nformatters:\n  settings:\n    list-marker:\n      marker: \"*\"\n");
	assert_eq!(
		output(&run(root.path(), &config, Mode::Lint)),
		"a.md:3: list item marked differently from the configured marker (list-marker)
\
		 a.md:4: list item marked differently from the configured marker (list-marker)
"
	);
	run(root.path(), &config, Mode::Format);
	assert_eq!(read(root.path(), "a.md"), "# A\n\n* one\n* two\n");
}

#[test]
fn parallel_runs_give_the_same_result() {
	let docs: Vec<(String, String)> = (0..200)
		.map(|i| {
			let next = i + 1;
			(
				format!("docs/d{i:03}.md"),
				format!("# Doc {i}\n\nGo -> [next](d{next:03}.md#doc-{next}) and [gone](x{i}.md)  \n"),
			)
		})
		.collect();
	let entries: Vec<(&str, &str)> = docs.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
	let root = tree(&entries);
	let first = output(&run(root.path(), &Config::default(), Mode::Lint));
	for _ in 0..5 {
		assert_eq!(output(&run(root.path(), &Config::default(), Mode::Lint)), first);
	}
	let lines: Vec<&str> = first.lines().collect();
	assert_eq!(
		lines.len(),
		601,
		"an arrow, trailing whitespace and a broken link in each of 200 documents, and the last one's link to a 201st"
	);
	assert!(
		lines.contains(&"docs/d000.md:3: broken link: x0.md (links)")
			&& lines.contains(&"docs/d199.md:3: broken link: d200.md#doc-200 (links)")
	);
}

#[test]
fn a_removed_file_is_missing_in_the_next_run() {
	let root = tree(&[
		("a.md", "# A\n\nSee [the logo](logo.png) and [b](b.md).\n"),
		("b.md", "# B\n"),
		("logo.png", "x"),
	]);
	let config = Config::default();
	let runner = Runner {
		root: root.path(),
		config: &config,
		mode: Mode::Lint,
	};
	assert_eq!(runner.run(&AtomicBool::new(false)).unwrap().issues, []);
	for name in ["logo.png", "b.md"] {
		fs::remove_file(root.path().join(name)).unwrap();
	}
	let second = runner.run(&AtomicBool::new(false)).unwrap();
	let messages: Vec<&str> = second.issues.iter().map(|i| i.message.as_str()).collect();
	assert_eq!(messages, ["broken link: b.md", "broken link: logo.png"]);
}

#[test]
fn a_stopped_run_writes_nothing() {
	let root = tree(&[("a.md", "a -> b\n")]);
	let config = Config::default();
	let result = Runner {
		root: root.path(),
		config: &config,
		mode: Mode::Format,
	}
	.run(&AtomicBool::new(true));
	assert!(matches!(result, Err(Error::Stopped)));
	assert_eq!(read(root.path(), "a.md"), "a -> b\n");
}

#[test]
fn a_document_that_is_not_utf8_is_an_error() {
	let root = tree(&[]);
	fs::write(root.path().join("a.md"), b"\xff\n").unwrap();
	let result = Runner {
		root: root.path(),
		config: &Config::default(),
		mode: Mode::Lint,
	}
	.run(&AtomicBool::new(false));
	assert!(result.unwrap_err().to_string().starts_with("reading a.md: "));
}
