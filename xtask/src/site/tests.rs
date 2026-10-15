use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::{check, documents, markup, rules};

fn pages() -> BTreeMap<String, String> {
    let root = crate::root();
    let mut pages = documents::pages(&root).unwrap();
    pages.extend(rules::pages(&root).unwrap());
    pages
}

#[test]
fn every_document_has_its_page() {
    let pages = pages();
    for path in [
        "docs/_index.md",
        "docs/rules.md",
        "docs/configuration.md",
        "docs/glossary.md",
        "docs/standards.md",
        "docs/mechanisms/_index.md",
        "docs/mechanisms/anchors.md",
        "docs/linters/_index.md",
        "docs/linters/index/index.md",
        "docs/formatters/arrows/index.md",
    ] {
        assert!(pages.contains_key(path), "{path}");
    }
}

#[test]
fn a_document_page_points_its_links_at_the_site() {
    let pages = pages();
    let start = &pages["docs/_index.md"];
    assert!(
        start.starts_with("+++\ntitle = \"Getting started\"\nweight = 1\n"),
        "{start}"
    );
    assert!(start.contains("navigation = [\"docs/_index.md\", \"docs/rules.md\", \"docs/linters/_index.md\""));
    assert!(start.contains("[Rules](@/docs/rules.md)"), "a document of the site");
    assert!(
        start.contains("(https://github.com/vertyll/markright/blob/main/LICENSE)"),
        "a file that is not a page"
    );
    assert!(
        start.contains("## Install {#install}"),
        "headings keep the anchors the documents link to"
    );
    assert!(!start.contains("# markright"), "the title is the page's");
    assert!(!start.contains("## Documentation"), "the navigation replaces it");
    assert!(start.contains("## License {#license}"), "the sections after it stay");
    assert!(
        pages["docs/mechanisms/anchors.md"]
            .contains("description = \"How headings and HTML ids become anchors and how fragments are matched.\""),
        "a page says what it covers, as the index does"
    );
    let rules = &pages["docs/rules.md"];
    assert!(
        rules.contains("[`links`](@/docs/linters/links/index.md)"),
        "rule names link to their pages"
    );
    assert!(rules.contains("[`arrows`](@/docs/formatters/arrows/index.md)"));
}

#[test]
fn a_rule_page_shows_its_example_settings_and_switch() {
    let pages = pages();
    let linter = &pages["docs/linters/line-length/index.md"];
    for part in [
        "title = \"line-length\"",
        "kind = \"Linter\"",
        "settings = true",
        "repository = false",
        "`markright check` reports:",
        "doc.md:1: line is 129 characters, limit 120 (line-length)",
        "Fixed:",
        "| `max` |",
        "      max: 120\n",
        "[linters.settings.line-length]\nmax = 120\n",
        "linters:\n  disable: [line-length]\n",
        "[linters]\ndisable = [\"line-length\"]\n",
        "More in [Rules](@/docs/rules.md#linters).",
    ] {
        assert!(linter.contains(part), "{part:?} in\n{linter}");
    }
    let formatter = &pages["docs/formatters/list-marker/index.md"];
    assert!(formatter.contains("autofix = true") && formatter.contains("`markright fmt` rewrites it to:"));
    assert!(
        formatter.contains("      marker: \"-\"\n"),
        "a value YAML would misread is quoted"
    );
    let index = &pages["docs/linters/index/index.md"];
    assert!(index.contains("repository = true") && index.contains("in the repository, `markright check` reports:"));
}

#[test]
fn a_fence_is_longer_than_the_backticks_inside() {
    assert_eq!(markup::fence("text", "a\n"), "```text\na\n```\n\n");
    assert_eq!(
        markup::fence("markdown", "```\nx\n```"),
        "````markdown\n```\nx\n```\n````\n\n"
    );
    assert_eq!(markup::string("say \"hi\"\n"), "\"say \\\"hi\\\"\\n\"");
}

fn site(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let file = dir.path().join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, content).unwrap();
    }
    dir
}

fn broken(public: &Path) -> Vec<String> {
    check::broken(public, "https://example.com/base/").unwrap()
}

#[test]
fn links_inside_the_site_must_lead_somewhere() {
    let public = site(&[
        (
            "index.html",
            r##"<a href="https://example.com/base/docs/#intro">ok</a>
			<a href=/base/docs/>ok</a>
			<a href="docs/">ok</a>
			<a href="#top">self</a><p id=top></p>
			<img src="/base/logo.svg?h=1">
			<a href="https://elsewhere.org/x">other site</a>
			<a href="mailto:x@example.com">mail</a>
			<a href="https://example.com/base/missing/">missing</a>
			<a href="docs/#nowhere">no anchor</a>
			<a href="/other/">outside</a>
			<a href="../up/">outside</a>"##,
        ),
        ("docs/index.html", r#"<h2 id="intro">Intro</h2><a href="../">up</a>"#),
        ("logo.svg", "<svg/>"),
    ]);
    assert_eq!(
        broken(public.path()),
        [
            "index.html: no page for /base/missing/",
            "index.html: no anchor for docs/#nowhere",
            "index.html: leaves the site, which is served under /base/: /other/",
            "index.html: leaves the site, which is served under /base/: ../up/",
        ]
    );
    assert!(check::check(public.path(), "https://example.com/base/").is_err());
}

#[test]
fn a_site_whose_links_resolve_passes() {
    let public = site(&[
        ("index.html", "<a href=a.html>a</a>"),
        ("a.html", "<a href=index.html>back</a>"),
    ]);
    assert_eq!(broken(public.path()), Vec::<String>::new());
    check::check(public.path(), "https://example.com/base/").unwrap();
}

#[test]
fn diagrams_become_pictures_in_both_themes() {
    let pages = pages();
    let architecture = &pages["docs/architecture.md"];
    assert!(!architecture.contains("```mermaid"), "{architecture}");
    assert_eq!(architecture.matches("<figure class=\"diagram\">").count(), 2);
    assert_eq!(architecture.matches("<div class=\"diagram-light\"><svg").count(), 2);
    assert_eq!(architecture.matches("<div class=\"diagram-dark\"><svg").count(), 2);
    assert!(
        architecture.contains("docs-architecture-md-diagram-1-dark"),
        "each picture has ids of its own"
    );
    let figure = architecture
        .lines()
        .find(|l| l.starts_with("<figure"))
        .unwrap_or_default();
    assert!(
        figure.ends_with("</figure>"),
        "a picture is one line, so Markdown keeps it a single HTML block"
    );
}
