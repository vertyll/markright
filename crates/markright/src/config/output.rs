//! Where `markright run` writes issues.

use std::path::PathBuf;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Where `markright run` writes issues.
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
	/// Whether no format is set, so text goes to the standard output.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.text.is_none() && self.checkstyle.is_none() && self.sarif.is_none()
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
	fn schema_name() -> std::borrow::Cow<'static, str> {
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
