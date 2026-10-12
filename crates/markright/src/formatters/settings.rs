//! The settings of the formatters, as `formatters.settings` of the configuration writes them: one entry per formatter
//! that takes any, each with its defaults.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The settings of the formatters, by formatter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
#[schemars(rename = "FormatterSettings")]
pub struct Settings {
	/// The settings of list-marker.
	pub list_marker: ListMarker,
}

/// The settings of list-marker.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ListMarker {
	/// The marker of every item of an unordered list.
	pub marker: Marker,
}

/// A marker `CommonMark` allows for an unordered list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Marker {
	/// `-`
	#[default]
	#[serde(rename = "-")]
	Dash,
	/// `*`
	#[serde(rename = "*")]
	Star,
	/// `+`
	#[serde(rename = "+")]
	Plus,
}

impl Marker {
	/// The marker.
	#[must_use]
	pub fn as_char(self) -> char {
		match self {
			Self::Dash => '-',
			Self::Star => '*',
			Self::Plus => '+',
		}
	}

	/// The marker as written.
	#[must_use]
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Dash => "-",
			Self::Star => "*",
			Self::Plus => "+",
		}
	}
}
