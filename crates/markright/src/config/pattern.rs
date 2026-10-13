//! A regular expression the configuration file gives, compiled as the file is read.

use std::borrow::Cow;

use regex::Regex;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A regular expression from the file, matched anywhere in a path or a message.
#[derive(Debug, Clone)]
pub struct Pattern(Regex);

impl Pattern {
	/// Whether the expression matches anywhere in `text`.
	pub(super) fn matches(&self, text: &str) -> bool {
		self.0.is_match(text)
	}

	/// The expression as the file writes it.
	pub(super) fn as_str(&self) -> &str {
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
	fn schema_name() -> Cow<'static, str> {
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
