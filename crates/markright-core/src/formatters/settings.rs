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
    /// The settings of ordered-list-numbers.
    pub ordered_list_numbers: OrderedListNumbers,
    /// The settings of emphasis-marker.
    pub emphasis_marker: EmphasisMarker,
    /// The settings of fence-marker.
    pub fence_marker: FenceMarker,
    /// The settings of horizontal-rule.
    pub horizontal_rule: HorizontalRule,
}

/// The settings of list-marker.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ListMarker {
    /// The marker of every item of an unordered list.
    pub marker: Marker,
}

/// The settings of ordered-list-numbers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct OrderedListNumbers {
    /// How the items of an ordered list are numbered.
    pub numbering: Numbering,
}

/// How the items of an ordered list are numbered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Numbering {
    /// Each item one more than the item before it, from the number of the first.
    #[default]
    Increasing,
    /// Every item with the number of the first.
    Same,
}

/// The settings of emphasis-marker.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct EmphasisMarker {
    /// The character of emphasis and strong emphasis.
    pub marker: Delimiter,
}

/// The character emphasis is written with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Delimiter {
    /// `*text*` and `**text**`.
    #[default]
    #[serde(rename = "*")]
    Asterisk,
    /// `_text_` and `__text__`.
    #[serde(rename = "_")]
    Underscore,
}

impl Delimiter {
    /// The character.
    #[must_use]
    pub fn as_char(self) -> char {
        match self {
            Self::Asterisk => '*',
            Self::Underscore => '_',
        }
    }
}

/// The settings of fence-marker.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct FenceMarker {
    /// The character of the fences of code blocks.
    pub marker: Fence,
}

/// The character a code fence is written with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Fence {
    /// ```` ``` ````
    #[default]
    #[serde(rename = "`")]
    Backtick,
    /// `~~~`
    #[serde(rename = "~")]
    Tilde,
}

impl Fence {
    /// The character.
    #[must_use]
    pub fn as_char(self) -> char {
        match self {
            Self::Backtick => '`',
            Self::Tilde => '~',
        }
    }
}

/// The settings of horizontal-rule.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct HorizontalRule {
    /// How a horizontal rule is written.
    pub style: Rule,
}

/// How a horizontal rule is written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Rule {
    /// `---`
    #[default]
    #[serde(rename = "---")]
    Dashes,
    /// `***`
    #[serde(rename = "***")]
    Asterisks,
    /// `___`
    #[serde(rename = "___")]
    Underscores,
}

impl Rule {
    /// The rule as written.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dashes => "---",
            Self::Asterisks => "***",
            Self::Underscores => "___",
        }
    }
}
