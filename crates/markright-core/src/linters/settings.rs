//! The settings of the linters, as `linters.settings` of the configuration writes them: one entry per linter that
//! takes any, each with its defaults.

use std::num::NonZeroUsize;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::INDEX_DOCUMENT;

/// The settings of the linters, by linter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
#[schemars(rename = "LinterSettings")]
pub struct Settings {
    /// The settings of line-length.
    pub line_length: LineLength,
    /// The settings of index.
    pub index: Index,
    /// The settings of readme-links.
    pub readme_links: ReadmeLinks,
    /// The settings of alerts.
    pub alerts: Alerts,
    /// The settings of link-text.
    pub link_text: LinkText,
}

/// The settings of line-length.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LineLength {
    /// The longest line allowed, in characters.
    pub max: NonZeroUsize,
}

impl Default for LineLength {
    fn default() -> Self {
        Self {
            max: NonZeroUsize::new(120).expect("120 is not zero"),
        }
    }
}

/// The settings of index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Index {
    /// The document that must link every other one.
    pub file: String,
}

impl Default for Index {
    fn default() -> Self {
        Self {
            file: INDEX_DOCUMENT.to_owned(),
        }
    }
}

/// The settings of readme-links.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ReadmeLinks {
    /// The README.
    pub file: String,
    /// Documents the README must link, when they exist.
    pub links: Vec<String>,
}

impl Default for ReadmeLinks {
    fn default() -> Self {
        let links = [INDEX_DOCUMENT, "GLOSSARY.md", "STANDARDS.md"]
            .map(str::to_owned)
            .to_vec();
        Self {
            file: "README.md".to_owned(),
            links,
        }
    }
}

/// The settings of alerts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Alerts {
    /// Allowed alert types.
    pub types: Vec<String>,
}

impl Default for Alerts {
    fn default() -> Self {
        Self {
            types: ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"]
                .map(str::to_owned)
                .to_vec(),
        }
    }
}

/// The settings of link-text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LinkText {
    /// The texts that say nothing about where a link leads, compared without case and without the punctuation around
    /// them.
    pub phrases: Vec<String>,
}

impl Default for LinkText {
    fn default() -> Self {
        Self {
            phrases: ["click here", "here", "link", "more", "read more", "this", "this link"]
                .map(str::to_owned)
                .to_vec(),
        }
    }
}
