//! A page for every formatter and linter: what it does, its example as markright checks it, its settings and how to turn
//! it off; and the sections that list them.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::io;
use std::path::Path;

use markright_core::config::{self, Config, Syntax};
use markright_core::runner::{self, Checked, Mode};
use markright_core::{formatters, linters, markdown, output};
use serde_json::Value;

use super::documents::{Section, section_page};
use super::markup::{fence, front_matter, string};
use crate::schema::{self, Setting};

/// The sections that list the rules, the formatters' and the linters'.
pub(super) const SECTIONS: [Section; 2] = [
    Section {
        target: "docs/linters/_index.md",
        title: "Linters",
        weight: 3,
        intro: "`markright check` runs every enabled linter and reports what it finds.",
        template: "rules.html",
        page_template: "rule.html",
    },
    Section {
        target: "docs/formatters/_index.md",
        title: "Formatters",
        weight: 4,
        intro: "`markright fmt` applies every enabled formatter, in this order, and `markright check` reports what each \
		        would change. Every formatter fixes what it finds.",
        template: "rules.html",
        page_template: "rule.html",
    },
];

/// The kind of a rule, which names its section of the configuration and of the site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Formatter,
    Linter,
}

impl Kind {
    /// The section of the configuration and of the site: `formatters` or `linters`.
    pub(super) const fn section(self) -> &'static str {
        match self {
            Self::Formatter => "formatters",
            Self::Linter => "linters",
        }
    }

    /// The kind as a page names it.
    const fn title(self) -> &'static str {
        match self {
            Self::Formatter => "Formatter",
            Self::Linter => "Linter",
        }
    }

    /// The other kind, whose rules an example turns off.
    const fn other(self) -> Self {
        match self {
            Self::Formatter => Self::Linter,
            Self::Linter => Self::Formatter,
        }
    }
}

/// What a rule's example shows: the document, the other files of its repository, what `markright check` reports and
/// the document fixed.
struct Example {
    path: &'static str,
    files: Vec<&'static str>,
    before: &'static str,
    report: String,
    after: String,
}

/// A formatter or a linter, as its page shows it.
struct Rule<'a> {
    kind: Kind,
    name: &'static str,
    description: &'static str,
    since: &'static str,
    weight: usize,
    repository: bool,
    settings: Vec<&'a Setting>,
    example: Example,
}

/// The page of every rule and of the sections that list them, by path in the content.
pub(super) fn pages(root: &Path) -> io::Result<BTreeMap<String, String>> {
    let settings = schema::settings(root)?;
    let rules = rules(&settings)?;
    let latest = rules.iter().map(|rule| release(rule.since)).max().unwrap_or_default();
    let mut pages: BTreeMap<String, String> = SECTIONS
        .iter()
        .map(|section| (section.target.to_owned(), section_page(section)))
        .collect();
    for rule in &rules {
        pages.insert(page(rule.kind, rule.name), rule.page(release(rule.since) == latest));
    }
    Ok(pages)
}

/// Every formatter, then every linter, each with its settings among `settings` and its example as markright checks it.
fn rules(settings: &[Setting]) -> io::Result<Vec<Rule<'_>>> {
    let of = |kind: Kind, name: &str| {
        settings
            .iter()
            .filter(|setting| setting.section == kind.section() && setting.rule == name)
            .collect()
    };
    let mut rules = Vec::new();
    for (i, formatter) in formatters::ALL.iter().enumerate() {
        let about = formatter.about();
        let document = BTreeMap::from([(DOCUMENT.to_owned(), about.example.to_owned())]);
        let checked = run(Kind::Formatter, about.name, &document)?;
        let after = checked
            .fixed
            .get(DOCUMENT)
            .cloned()
            .unwrap_or_else(|| about.example.to_owned());
        rules.push(Rule {
            kind: Kind::Formatter,
            name: about.name,
            description: about.description,
            since: about.since,
            weight: i + 1,
            repository: false,
            settings: of(Kind::Formatter, about.name),
            example: Example {
                path: DOCUMENT,
                files: Vec::new(),
                before: about.example,
                report: output::text(&checked.issues),
                after,
            },
        });
    }
    for (i, linter) in linters::ALL.iter().enumerate() {
        let about = linter.about();
        let example = about.example;
        let mut documents = BTreeMap::from([(example.path.to_owned(), example.bad.to_owned())]);
        documents.extend(
            example
                .files
                .iter()
                .map(|&(path, text)| (path.to_owned(), text.to_owned())),
        );
        let checked = run(Kind::Linter, about.name, &documents)?;
        rules.push(Rule {
            kind: Kind::Linter,
            name: about.name,
            description: about.description,
            since: about.since,
            weight: i + 1,
            repository: linter.is_repository(),
            settings: of(Kind::Linter, about.name),
            example: Example {
                path: example.path,
                files: example.files.iter().map(|&(path, _)| path).collect(),
                before: example.bad,
                report: output::text(&checked.issues),
                after: example.good.to_owned(),
            },
        });
    }
    Ok(rules)
}

/// The page of the rule `name` of `kind` in the content: a directory of its own, so a rule may be called `index` or
/// share its name with any page.
pub(super) fn page(kind: Kind, name: &str) -> String {
    format!("docs/{}/{name}/index.md", kind.section())
}

/// Where an example's document lives when it says nothing else.
const DOCUMENT: &str = "doc.md";

/// Runs `markright check` on `documents`, held in memory, with only the rule `name` of `kind` turned on.
fn run(kind: Kind, name: &str, documents: &BTreeMap<String, String>) -> io::Result<Checked> {
    let (section, other) = (kind.section(), kind.other().section());
    let text = format!("version: \"1\"\n{section}:\n  default: none\n  enable: [{name}]\n{other}:\n  default: none\n");
    let config = Config::parse(&text, config::YAML_FILE, Syntax::Yaml).map_err(io::Error::other)?;
    runner::check_documents(documents, &config, Mode::Lint).map_err(io::Error::other)
}

/// The parts of a release such as `v0.1.0`, to compare releases by.
fn release(name: &str) -> Vec<u32> {
    name.trim_start_matches('v')
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect()
}

impl Rule<'_> {
    /// Whether `markright fmt` fixes what the rule finds: every formatter does.
    fn autofix(&self) -> bool {
        self.kind == Kind::Formatter
    }

    /// The rule's page: front matter for the lists and the badges, then the example, the settings and how to turn
    /// it off.
    fn page(&self, new: bool) -> String {
        let description = self.description.trim_end_matches('.');
        let fields = [
            ("title", string(self.name)),
            ("description", string(&markdown::plain(description))),
            ("weight", self.weight.to_string()),
        ];
        let extra = [
            ("kind", string(self.kind.title())),
            ("summary", string(description)),
            ("since", string(self.since)),
            ("new", new.to_string()),
            ("autofix", self.autofix().to_string()),
            ("settings", (!self.settings.is_empty()).to_string()),
            ("repository", self.repository.to_string()),
        ];
        let mut text = front_matter(&fields, &extra);
        let _ = write!(text, "{}: {description}.\n\n", self.kind.title());
        self.write_example(&mut text);
        self.write_settings(&mut text);
        let (section, name) = (self.kind.section(), self.name);
        text.push_str("## Turning it off {#turning-it-off}\n\n");
        text.push_str(&fence("yaml", &format!("{section}:\n  disable: [{name}]\n")));
        text.push_str(&fence("toml", &format!("[{section}]\ndisable = [{}]\n", string(name))));
        let _ = writeln!(text, "More in [Rules](@/docs/rules.md#{section}).");
        text
    }

    /// Adds the example to `text`: the document, what `markright check` reports and the document fixed.
    fn write_example(&self, text: &mut String) {
        let example = &self.example;
        text.push_str("## Example {#example}\n\n");
        let _ = write!(text, "`{}`:\n\n", example.path);
        text.push_str(&fence("markdown", example.before));
        if example.files.is_empty() {
            text.push_str("`markright check` reports:\n\n");
        } else {
            let files: Vec<String> = example.files.iter().map(|file| format!("`{file}`")).collect();
            let _ = write!(
                text,
                "With {} in the repository, `markright check` reports:\n\n",
                files.join(" and ")
            );
        }
        text.push_str(&fence("text", &example.report));
        text.push_str(if self.autofix() {
            "`markright fmt` rewrites it to:\n\n"
        } else {
            "Fixed:\n\n"
        });
        text.push_str(&fence("markdown", &example.after));
    }

    /// Adds the settings to `text`, when the rule takes any: each key, and the defaults in YAML and in TOML.
    fn write_settings(&self, text: &mut String) {
        if self.settings.is_empty() {
            return;
        }
        let (section, name) = (self.kind.section(), self.name);
        text.push_str("## Settings {#settings}\n\n");
        let _ = write!(text, "Under `{section}.settings.{name}`:\n\n");
        text.push_str("| Key | Meaning | Default |\n|---|---|---|\n");
        for setting in &self.settings {
            let meaning = setting.description.trim_end_matches('.');
            let _ = writeln!(
                text,
                "| `{}` | {meaning} | {} |",
                setting.key,
                crate::tables::default(&setting.default)
            );
        }
        text.push_str("\nWith the defaults, in `.markright.yaml`:\n\n");
        let mut yaml = format!("{section}:\n  settings:\n    {name}:\n");
        let mut toml = format!("[{section}.settings.{name}]\n");
        for setting in &self.settings {
            let _ = writeln!(yaml, "      {}: {}", setting.key, yaml_value(&setting.default));
            let _ = writeln!(toml, "{} = {}", setting.key, setting.default);
        }
        text.push_str(&fence("yaml", &yaml));
        text.push_str("or in `.markright.toml`:\n\n");
        text.push_str(&fence("toml", &toml));
    }
}

/// `value` as YAML: a list in brackets, and a string in quotes unless YAML reads it as itself, so a value such as `-`
/// or `*` is not taken for a list or an alias.
fn yaml_value(value: &Value) -> String {
    match value {
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(yaml_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::String(text) if is_plain(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Whether YAML reads `text` as itself without quotes: a word, a path or a file name.
fn is_plain(text: &str) -> bool {
    text.chars().next().is_some_and(|first| first.is_ascii_alphanumeric())
        && text
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_./-".contains(character))
}
