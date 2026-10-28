//! Runs every rule over the examples of the specifications the parser follows: the 652 of `CommonMark` 0.31.2 and the
//! 23 of the GFM 0.29 extensions markright reads, tables, strikethrough, autolinks and task lists. The examples are the
//! cases the specifications found worth spelling out, the odd ones most of all, so they are where a formatter would
//! change what a document says without meaning to.
//!
//! For each example, every formatter that only changes how a document is written leaves the HTML it renders to as it
//! was, alone and with all of them in a row as `markright fmt` runs them; every formatter leaves what it made as it is;
//! and the whole pass, formatters and linters, runs through it. The examples are in `tests/spec`, taken as written from
//! the specifications, which are published under CC BY-SA 4.0; each file names its source, authors, license and what
//! was left out. They stay out of the published crate.

use std::collections::BTreeMap;

use markright_core::config::Config;
use markright_core::formatters::{ALL, Formatter, Settings};
use markright_core::markdown::Document;
use markright_core::runner::{Mode, check_documents};
use serde::Deserialize;

/// A file of examples.
#[derive(Deserialize)]
struct Examples {
    examples: Vec<Example>,
}

/// One example of a specification: its number there, its section and its Markdown.
#[derive(Deserialize)]
struct Example {
    #[serde(rename = "example")]
    number: u32,
    section: String,
    markdown: String,
}

/// The formatters that change what a document says, not only how it is written: they are left out of the check that
/// the HTML stays the same.
const CHANGE_THE_TEXT: [&str; 4] = ["trailing-whitespace", "arrows", "link-spacing", "bare-urls"];

/// Every example of both specifications, each with the name of its specification.
fn examples() -> Vec<(&'static str, Example)> {
    let read = |name: &'static str, json: &str| {
        let file: Examples = serde_json::from_str(json).expect("the examples are JSON");
        file.examples.into_iter().map(move |example| (name, example))
    };
    read("CommonMark", include_str!("spec/commonmark-0.31.2.json"))
        .chain(read("GFM", include_str!("spec/gfm-0.29-extensions.json")))
        .collect()
}

/// The HTML `text` renders to, with the extensions `markright-core` parses with, its runs of white space made one space, as a
/// browser shows them, and none at the end of a code block.
fn rendered(text: &str) -> String {
    let mut html = String::new();
    let options = pulldown_cmark::Options::ENABLE_TABLES
        | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
        | pulldown_cmark::Options::ENABLE_TASKLISTS
        | pulldown_cmark::Options::ENABLE_FOOTNOTES
        | pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
    pulldown_cmark::html::push_html(&mut html, pulldown_cmark::Parser::new_ext(text, options));
    html.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" </code></pre>", "</code></pre>")
}

/// What `formatter` makes of `text`.
fn format(formatter: &dyn Formatter, text: &str) -> String {
    formatter
        .apply(&Document::parse(text), &Settings::default())
        .lines
        .join("\n")
}

/// What each failing example was, one per line, to read every failure at once rather than the first.
fn report(failures: &[String]) -> String {
    format!("{} failing:\n{}", failures.len(), failures.join("\n"))
}

#[test]
fn every_example_is_read() {
    assert_eq!(examples().len(), 652 + 23);
}

#[test]
fn formatters_keep_what_each_example_renders_to() {
    let mut failures = Vec::new();
    for (spec, example) in examples() {
        let text = &example.markdown;
        for formatter in ALL.iter().filter(|f| !CHANGE_THE_TEXT.contains(&f.about().name)) {
            if rendered(&format(*formatter, text)) != rendered(text) {
                failures.push(format!(
                    "{spec} {} ({}): {} on {text:?}",
                    example.number,
                    example.section,
                    formatter.about().name
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn the_formatters_in_a_row_keep_what_each_example_renders_to() {
    let mut failures = Vec::new();
    for (spec, example) in examples() {
        let text = &example.markdown;
        let formatted = ALL
            .iter()
            .filter(|f| !CHANGE_THE_TEXT.contains(&f.about().name))
            .fold(text.clone(), |text, formatter| format(*formatter, &text));
        if rendered(&formatted) != rendered(text) {
            failures.push(format!(
                "{spec} {} ({}): {text:?} became {formatted:?}",
                example.number, example.section
            ));
        }
    }
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn formatters_leave_what_they_made_of_each_example() {
    let mut failures = Vec::new();
    for (spec, example) in examples() {
        for formatter in ALL {
            let once = format(*formatter, &example.markdown);
            let twice = format(*formatter, &once);
            if twice != once {
                failures.push(format!(
                    "{spec} {} ({}): {} made {once:?}, then {twice:?}",
                    example.number,
                    example.section,
                    formatter.about().name
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn every_rule_runs_through_each_example() {
    for (spec, example) in examples() {
        let documents = BTreeMap::from([("doc.md".to_owned(), example.markdown.clone())]);
        for mode in [Mode::Lint, Mode::Fix] {
            check_documents(&documents, &Config::default(), mode)
                .unwrap_or_else(|error| panic!("{spec} {}: {error}", example.number));
        }
    }
}
