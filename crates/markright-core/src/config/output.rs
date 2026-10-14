//! Where `markright check` writes issues.

use std::borrow::Cow;
use std::fmt;
use std::path::PathBuf;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Where `markright check` writes issues.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
	/// One entry per output format.
	#[serde(default)]
	pub formats: Formats,
}

/// Where each output format goes; a format without one is not written. With none set, text goes to the standard
/// output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Formats {
	/// One issue per line, then a summary.
	#[serde(default)]
	pub text: Option<Format>,
	/// Checkstyle XML.
	#[serde(default)]
	pub checkstyle: Option<Format>,
	/// SARIF 2.1.0, for tools that read static analysis results.
	#[serde(default)]
	pub sarif: Option<Format>,
}

impl Default for Formats {
	/// Text to the standard output, as when the file sets no format.
	fn default() -> Self {
		Self {
			text: Some(Format {
				path: Destination::Stdout,
			}),
			checkstyle: None,
			sarif: None,
		}
	}
}

impl Formats {
	/// Where `format` goes, when the file sets it.
	#[must_use]
	pub fn get(&self, format: ReportFormat) -> Option<&Destination> {
		let format = match format {
			ReportFormat::Text => &self.text,
			ReportFormat::Checkstyle => &self.checkstyle,
			ReportFormat::Sarif => &self.sarif,
		};
		format.as_ref().map(|format| &format.path)
	}

	/// Whether no format is set, so text goes to the standard output.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		ReportFormat::ALL.iter().all(|&format| self.get(format).is_none())
	}
}

/// A format `markright check` can write issues in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReportFormat {
	/// One issue per line, then a summary.
	Text,
	/// Checkstyle XML.
	Checkstyle,
	/// SARIF 2.1.0.
	Sarif,
}

impl ReportFormat {
	/// Every format, in the order they are written.
	pub const ALL: [Self; 3] = [Self::Text, Self::Checkstyle, Self::Sarif];

	/// The format's key under `output.formats`.
	#[must_use]
	pub fn name(self) -> &'static str {
		match self {
			Self::Text => "text",
			Self::Checkstyle => "checkstyle",
			Self::Sarif => "sarif",
		}
	}
}

impl fmt::Display for ReportFormat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.name())
	}
}

/// Where one output format goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Format {
	/// stdout, stderr, or a file path relative to the working directory.
	pub path: Destination,
}

/// A standard stream or a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
	/// The standard output, written `stdout`.
	Stdout,
	/// The standard error, written `stderr`.
	Stderr,
	/// A file, created or replaced.
	File(PathBuf),
}

/// `stdout` and `stderr` name the streams, and anything else a file; an empty path is an error.
impl<'de> Deserialize<'de> for Destination {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		match String::deserialize(deserializer)?.as_str() {
			"" => Err(serde::de::Error::custom(
				"the path is empty; use stdout, stderr or a file",
			)),
			"stdout" => Ok(Self::Stdout),
			"stderr" => Ok(Self::Stderr),
			path => Ok(Self::File(path.into())),
		}
	}
}

impl JsonSchema for Destination {
	fn schema_name() -> Cow<'static, str> {
		"Destination".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({ "type": "string", "minLength": 1 })
	}
}

impl Serialize for Destination {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Stdout => serializer.serialize_str("stdout"),
			Self::Stderr => serializer.serialize_str("stderr"),
			Self::File(path) => path.serialize(serializer),
		}
	}
}
