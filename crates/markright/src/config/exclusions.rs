//! Which documents the formatters leave alone and which issues the linters drop.

use std::fmt;

use regex::Regex;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Config, schema};

/// A regular expression from the file, matched anywhere in a path or a message.
#[derive(Debug, Clone)]
pub struct Pattern(Regex);

impl Pattern {
	fn matches(&self, text: &str) -> bool {
		self.0.is_match(text)
	}

	fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

/// A pattern is read as it is compiled, so a wrong one is an error at its line of the file. An empty one, which would
/// match everything, is one too: a key that should not apply is left out.
impl<'de> Deserialize<'de> for Pattern {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let raw = String::deserialize(deserializer)?;
		if raw.is_empty() {
			return Err(serde::de::Error::custom(
				"empty regular expression; leave the key out instead",
			));
		}
		Regex::new(&raw).map(Self).map_err(|e| {
			// The parser's message draws the pattern over several lines; its last line says what is wrong.
			let reason = e
				.to_string()
				.lines()
				.last()
				.unwrap_or_default()
				.trim_start_matches("error: ")
				.to_owned();
			serde::de::Error::custom(format!("invalid regular expression {raw:?}: {reason}"))
		})
	}
}

impl PartialEq for Pattern {
	fn eq(&self, other: &Self) -> bool {
		self.as_str() == other.as_str()
	}
}

impl JsonSchema for Pattern {
	fn schema_name() -> std::borrow::Cow<'static, str> {
		"Pattern".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({ "type": "string", "format": "regex", "minLength": 1, "description": "A regular expression, matched anywhere." })
	}
}

impl Serialize for Pattern {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(self.as_str())
	}
}

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
pub enum Section {
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
pub enum ExclusionKey {
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
pub struct Matched {
	/// Whether the document or the issue is excluded.
	pub excluded: bool,
	/// The exclusions that matched it.
	pub by: Vec<ExclusionKey>,
}

/// An exclusion of a section that asks for warn-unused, and the warning printed when it matched nothing in a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watched {
	/// The exclusion.
	pub key: ExclusionKey,
	/// What to print when it matched nothing.
	pub warning: String,
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
				.filter(|(_, p)| p.matches(path))
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
		let warn = |key: ExclusionKey, p: &Pattern| Watched {
			key,
			warning: format!("{key}: {:?} matches no document", p.as_str()),
		};
		let paths = self
			.paths
			.iter()
			.enumerate()
			.map(move |(index, p)| warn(ExclusionKey::Path { section, index }, p));
		let except = self
			.except
			.iter()
			.enumerate()
			.map(move |(index, p)| warn(ExclusionKey::PathExcept { section, index }, p));
		paths.chain(except)
	}
}

impl LinterExclusions {
	fn paths(&self) -> PathFilter<'_> {
		PathFilter {
			section: Section::Linters,
			paths: &self.paths,
			except: &self.paths_except,
		}
	}
}

impl FormatterExclusions {
	fn paths(&self) -> PathFilter<'_> {
		PathFilter {
			section: Section::Formatters,
			paths: &self.paths,
			except: &self.paths_except,
		}
	}
}

impl LinterExclusions {
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
			if fields.into_iter().filter(|&f| f).count() < 2 {
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
	fn matches(&self, path: &str, linter: &str, message: &str) -> bool {
		self.path.as_ref().is_none_or(|p| p.matches(path))
			&& self.path_except.as_ref().is_none_or(|p| !p.matches(path))
			&& (self.linters.is_empty() || self.linters.iter().any(|l| l == linter))
			&& self.text.as_ref().is_none_or(|p| p.matches(message))
	}
}

impl Config {
	/// Whether the formatters leave the document at `path` alone, and the exclusions that matched it.
	#[must_use]
	pub fn formatter_excluded(&self, path: &str) -> Matched {
		self.formatters.exclusions.paths().exclude(path)
	}

	/// Whether the linter exclusion paths cover the document at `path`, and the exclusions that matched it.
	#[must_use]
	pub fn linter_path_excluded(&self, path: &str) -> Matched {
		self.linters.exclusions.paths().exclude(path)
	}

	/// Whether an issue `linter` found at `path`, with `message`, is dropped, and the exclusions that matched it.
	#[must_use]
	pub fn linter_excluded(&self, path: &str, linter: &str, message: &str) -> Matched {
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
	pub fn watched(&self) -> Vec<Watched> {
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
