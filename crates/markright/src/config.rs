//! Reads the configuration file, `.markright.yaml` or `.markright.toml`: which formatters and linters run, their
//! settings, their exclusions and where issues are written.

mod error;
mod exclusions;
mod output;
mod read;
mod schema;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use error::ConfigError;
pub use exclusions::{
	ExclusionKey, ExclusionRule, FormatterExclusions, LinterExclusions, Matched, Pattern, Section, Watched,
};
pub use output::{Destination, Format, Formats, Output, ReportFormat};
pub use read::Syntax;

use crate::{formatters, linters};

/// The configuration file in YAML, which the root may hold.
pub const YAML_FILE: &str = ".markright.yaml";
/// The configuration file in TOML, which the root may hold instead.
pub const TOML_FILE: &str = ".markright.toml";
/// The format the file must declare.
pub const VERSION: &str = "1";

/// The configuration file: which formatters and linters run, their settings, their exclusions and where issues are
/// written.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "markright configuration")]
pub struct Config {
	/// The configuration format version.
	#[schemars(schema_with = "schema::version")]
	pub version: String,
	/// Where markright run writes issues; text to stdout when no format is set.
	#[serde(default)]
	pub output: Output,
	/// The linters that run, their settings and exclusions.
	#[serde(default)]
	pub linters: Linters,
	/// The formatters that run, their settings and exclusions.
	#[serde(default)]
	pub formatters: Formatters,
}

/// The linters that run, their settings and which issues to drop.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Linters {
	/// The starting set: every linter, or none.
	#[serde(default)]
	pub default: Selection,
	/// Linters added to the default set.
	#[serde(default)]
	#[schemars(schema_with = "schema::linter_names")]
	pub enable: Vec<String>,
	/// Linters removed from the default set.
	#[serde(default)]
	#[schemars(schema_with = "schema::linter_names")]
	pub disable: Vec<String>,
	/// The settings of the linters that take any.
	#[serde(default)]
	pub settings: linters::Settings,
	/// The documents no linter reports on, and the issues to drop.
	#[serde(default)]
	pub exclusions: LinterExclusions,
}

/// The formatters that run, their settings and which documents they leave alone.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Formatters {
	/// The starting set: every formatter, or none.
	#[serde(default)]
	pub default: Selection,
	/// Formatters added to the default set.
	#[serde(default)]
	#[schemars(schema_with = "schema::formatter_names")]
	pub enable: Vec<String>,
	/// Formatters removed from the default set.
	#[serde(default)]
	#[schemars(schema_with = "schema::formatter_names")]
	pub disable: Vec<String>,
	/// The settings of the formatters that take any.
	#[serde(default)]
	pub settings: formatters::Settings,
	/// The documents the formatters leave as they are.
	#[serde(default)]
	pub exclusions: FormatterExclusions,
}

/// The starting set of a section: every rule, or none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Selection {
	/// Every rule runs unless disabled.
	#[default]
	All,
	/// Only the enabled rules run.
	None,
}

impl Default for Config {
	/// The configuration used when there is no file: every formatter and linter on, with its default settings.
	fn default() -> Self {
		Self {
			version: VERSION.to_owned(),
			output: Output::default(),
			linters: Linters::default(),
			formatters: Formatters::default(),
		}
	}
}

impl Config {
	/// The configuration file of the repository at `root`: `.markright.yaml` or `.markright.toml`, or none.
	///
	/// # Errors
	///
	/// Returns an error when both exist, as it would be unclear which one counts.
	pub fn locate(root: &Path) -> Result<Option<PathBuf>, ConfigError> {
		let found: Vec<PathBuf> = [YAML_FILE, TOML_FILE]
			.iter()
			.map(|name| root.join(name))
			.filter(|p| p.is_file())
			.collect();
		match <[PathBuf; 2]>::try_from(found) {
			Ok(_) => Err(ConfigError::Ambiguous { root: root.to_owned() }),
			Err(found) => Ok(found.into_iter().next()),
		}
	}

	/// Reads the configuration at `path`, in the language its extension names, falling back to the default when the
	/// file does not exist.
	///
	/// # Errors
	///
	/// Returns why the file could not be read, or what is wrong with it.
	pub fn load(path: &Path) -> Result<Self, ConfigError> {
		match fs::read_to_string(path) {
			Ok(text) => {
				let name = path
					.file_name()
					.map_or_else(|| path.display().to_string(), |n| n.display().to_string());
				Self::parse(&text, &name, Syntax::of(path))
			}
			Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
			Err(source) => Err(ConfigError::Read {
				path: path.display().to_string(),
				source,
			}),
		}
	}

	/// Reads a configuration from `text`, the content of the file called `file`, written in `syntax`, and checks it.
	///
	/// # Errors
	///
	/// Returns what is wrong with the configuration.
	pub fn parse(text: &str, file: &str, syntax: Syntax) -> Result<Self, ConfigError> {
		// An empty YAML file holds nothing at all; an empty TOML file is an empty table, which serde finds lacking.
		let missing = || ConfigError::Parse {
			file: file.to_owned(),
			line: 1,
			message: "missing field `version`".to_owned(),
		};
		let config: Self = read::parse(text, file, syntax)?.ok_or_else(missing)?;
		let invalid = |message: String| ConfigError::Invalid {
			file: file.to_owned(),
			message,
		};
		if config.version != VERSION {
			return Err(invalid(format!(
				"version must be {VERSION:?}, got {:?}",
				config.version
			)));
		}
		let (linters, formatters) = (schema::linter_names_list(), schema::formatter_names_list());
		check_selection("linters", &config.linters.enable, &config.linters.disable, &linters).map_err(invalid)?;
		check_selection(
			"formatters",
			&config.formatters.enable,
			&config.formatters.disable,
			&formatters,
		)
		.map_err(invalid)?;
		config.linters.exclusions.check(&linters).map_err(invalid)?;
		Ok(config)
	}

	/// Whether the linter called `name` runs.
	#[must_use]
	pub fn linter_enabled(&self, name: &str) -> bool {
		enabled(self.linters.default, &self.linters.enable, &self.linters.disable, name)
	}

	/// Whether the formatter called `name` runs.
	#[must_use]
	pub fn formatter_enabled(&self, name: &str) -> bool {
		enabled(
			self.formatters.default,
			&self.formatters.enable,
			&self.formatters.disable,
			name,
		)
	}
}

fn check_selection(section: &str, enable: &[String], disable: &[String], names: &[&str]) -> Result<(), String> {
	if let Some(name) = enable
		.iter()
		.chain(disable)
		.find(|name| !names.contains(&name.as_str()))
	{
		return Err(format!("{section}: unknown name {name:?} (see markright {section})"));
	}
	if let Some(name) = enable.iter().find(|name| disable.contains(name)) {
		return Err(format!("{section}: {name:?} is both enabled and disabled"));
	}
	Ok(())
}

fn enabled(default: Selection, enable: &[String], disable: &[String], name: &str) -> bool {
	let named = |list: &[String]| list.iter().any(|n| n == name);
	!named(disable) && (named(enable) || default == Selection::All)
}

#[cfg(test)]
mod tests;
