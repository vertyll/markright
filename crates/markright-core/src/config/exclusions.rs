//! Which documents the formatters leave alone and which issues the linters drop.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Config, Pattern, schema};

/// Which documents no linter reports on and which issues to drop.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct LinterExclusions {
    /// Warn about paths and rules that exclude nothing.
    #[serde(default)]
    pub warn_unused: bool,
    /// Documents no linter reports on.
    #[serde(default)]
    pub paths: Vec<Pattern>,
    /// When set, documents that match none of these are excluded.
    #[serde(default)]
    pub paths_except: Vec<Pattern>,
    /// Issues to drop.
    #[serde(default)]
    pub rules: Vec<ExclusionRule>,
}

/// Drops an issue that matches every field set; at least two of path or path-except, linters and text.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ExclusionRule {
    /// The document path matches.
    #[serde(default)]
    pub path: Option<Pattern>,
    /// The document path does not match.
    #[serde(default)]
    pub path_except: Option<Pattern>,
    /// The issue comes from one of these linters.
    #[serde(default)]
    #[schemars(schema_with = "schema::linter_names")]
    pub linters: Vec<String>,
    /// The issue message matches.
    #[serde(default)]
    pub text: Option<Pattern>,
}

/// Which documents the formatters leave as they are.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct FormatterExclusions {
    /// Warn about paths that exclude nothing.
    #[serde(default)]
    pub warn_unused: bool,
    /// Documents the formatters leave as they are.
    #[serde(default)]
    pub paths: Vec<Pattern>,
    /// When set, documents that match none of these are left as they are.
    #[serde(default)]
    pub paths_except: Vec<Pattern>,
}

/// A section of the file that has exclusions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Section {
    /// `formatters`
    Formatters,
    /// `linters`
    Linters,
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Formatters => "formatters",
            Self::Linters => "linters",
        })
    }
}

/// One exclusion of the file, as its key there names it, such as `linters.exclusions.paths[0]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ExclusionKey {
    /// `<section>.exclusions.paths[index]`
    Path {
        /// The section.
        section: Section,
        /// The pattern's place in the list.
        index: usize,
    },
    /// `<section>.exclusions.paths-except[index]`
    PathExcept {
        /// The section.
        section: Section,
        /// The pattern's place in the list.
        index: usize,
    },
    /// `linters.exclusions.rules[index]`
    Rule(usize),
}

impl fmt::Display for ExclusionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path { section, index } => write!(f, "{section}.exclusions.paths[{index}]"),
            Self::PathExcept { section, index } => write!(f, "{section}.exclusions.paths-except[{index}]"),
            Self::Rule(index) => write!(f, "linters.exclusions.rules[{index}]"),
        }
    }
}

/// What the exclusions make of a document or an issue: whether they exclude it, and which of them matched it, which
/// warn-unused counts as used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Matched {
    /// Whether the document or the issue is excluded.
    pub(crate) excluded: bool,
    /// The exclusions that matched it.
    pub(crate) by: Vec<ExclusionKey>,
}

/// An exclusion of a section that asks for warn-unused, and the warning printed when it matched nothing in a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Watched {
    /// The exclusion.
    pub(crate) key: ExclusionKey,
    /// What to print when it matched nothing.
    pub(crate) warning: String,
}

/// The path patterns of a section: those that exclude a document, and those that keep only the documents they
/// match.
struct PathFilter<'a> {
    section: Section,
    paths: &'a [Pattern],
    except: &'a [Pattern],
}

impl PathFilter<'_> {
    /// Whether the patterns exclude `path`, and the keys of those that matched it. With `except` set, a path it does
    /// not match is excluded too.
    fn exclude(&self, path: &str) -> Matched {
        let section = self.section;
        let matching = |patterns: &[Pattern]| -> Vec<usize> {
            patterns
                .iter()
                .enumerate()
                .filter(|(_, pattern)| pattern.matches(path))
                .map(|(index, _)| index)
                .collect()
        };
        let (hits, kept) = (matching(self.paths), matching(self.except));
        let excluded = !hits.is_empty() || (!self.except.is_empty() && kept.is_empty());
        let keys = hits
            .into_iter()
            .map(|index| ExclusionKey::Path { section, index })
            .chain(
                kept.into_iter()
                    .map(|index| ExclusionKey::PathExcept { section, index }),
            );
        Matched {
            excluded,
            by: keys.collect(),
        }
    }

    /// Every pattern, with the warning printed when it matched nothing.
    fn watched(&self) -> impl Iterator<Item = Watched> {
        let section = self.section;
        let warn = |key: ExclusionKey, pattern: &Pattern| Watched {
            key,
            warning: format!("{key}: {:?} matches no document", pattern.as_str()),
        };
        let paths = self
            .paths
            .iter()
            .enumerate()
            .map(move |(index, pattern)| warn(ExclusionKey::Path { section, index }, pattern));
        let except = self
            .except
            .iter()
            .enumerate()
            .map(move |(index, pattern)| warn(ExclusionKey::PathExcept { section, index }, pattern));
        paths.chain(except)
    }
}

impl FormatterExclusions {
    /// The section's path patterns.
    fn paths(&self) -> PathFilter<'_> {
        PathFilter {
            section: Section::Formatters,
            paths: &self.paths,
            except: &self.paths_except,
        }
    }
}

impl LinterExclusions {
    /// The section's path patterns.
    fn paths(&self) -> PathFilter<'_> {
        PathFilter {
            section: Section::Linters,
            paths: &self.paths,
            except: &self.paths_except,
        }
    }

    /// Checks the rules against `linters`, the names of every linter: each sets at least two fields, never both
    /// paths, and names only linters that exist.
    pub(super) fn check(&self, linters: &[&str]) -> Result<(), String> {
        for (i, rule) in self.rules.iter().enumerate() {
            let key = ExclusionKey::Rule(i);
            if rule.path.is_some() && rule.path_except.is_some() {
                return Err(format!("{key}: path and path-except cannot both be set"));
            }
            let fields = [
                rule.path.is_some() || rule.path_except.is_some(),
                !rule.linters.is_empty(),
                rule.text.is_some(),
            ];
            if fields.into_iter().filter(|&set| set).count() < 2 {
                return Err(format!(
                    "{key}: set at least two of path or path-except, linters and text"
                ));
            }
            if let Some(name) = rule.linters.iter().find(|name| !linters.contains(&name.as_str())) {
                return Err(format!("{key}: unknown linter {name:?}"));
            }
        }
        Ok(())
    }
}

impl ExclusionRule {
    /// Whether the issue `linter` found at `path`, with `message`, matches every field the rule sets.
    fn matches(&self, path: &str, linter: &str, message: &str) -> bool {
        self.path.as_ref().is_none_or(|pattern| pattern.matches(path))
            && self.path_except.as_ref().is_none_or(|pattern| !pattern.matches(path))
            && (self.linters.is_empty() || self.linters.iter().any(|name| name == linter))
            && self.text.as_ref().is_none_or(|pattern| pattern.matches(message))
    }
}

impl Config {
    /// Whether the formatters leave the document at `path` alone, and the exclusions that matched it.
    #[must_use]
    pub(crate) fn formatter_excluded(&self, path: &str) -> Matched {
        self.formatters.exclusions.paths().exclude(path)
    }

    /// Whether the linter exclusion paths cover the document at `path`, and the exclusions that matched it.
    #[must_use]
    pub(crate) fn linter_path_excluded(&self, path: &str) -> Matched {
        self.linters.exclusions.paths().exclude(path)
    }

    /// Whether an issue `linter` found at `path`, with `message`, is dropped, and the exclusions that matched it.
    #[must_use]
    pub(crate) fn linter_excluded(&self, path: &str, linter: &str, message: &str) -> Matched {
        let mut matched = self.linter_path_excluded(path);
        let rules = &self.linters.exclusions.rules;
        if let Some(i) = rules.iter().position(|rule| rule.matches(path, linter, message)) {
            matched.excluded = true;
            matched.by.push(ExclusionKey::Rule(i));
        }
        matched
    }

    /// The exclusions of the sections that ask for warn-unused, in the order the file declares them.
    #[must_use]
    pub(crate) fn watched(&self) -> Vec<Watched> {
        let mut out = Vec::new();
        let formatters = &self.formatters.exclusions;
        if formatters.warn_unused {
            out.extend(formatters.paths().watched());
        }
        let linters = &self.linters.exclusions;
        if linters.warn_unused {
            out.extend(linters.paths().watched());
            out.extend((0..linters.rules.len()).map(|i| {
                let key = ExclusionKey::Rule(i);
                Watched {
                    key,
                    warning: format!("{key} matches no issue"),
                }
            }));
        }
        out
    }
}
