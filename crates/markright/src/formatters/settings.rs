//! The settings of the formatters, as `formatters.settings` of the configuration writes them: one entry per formatter
//! that takes any, each with its defaults.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::markdown::Marker;

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
