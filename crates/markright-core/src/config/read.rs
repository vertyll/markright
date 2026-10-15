//! Reads the file, in YAML or TOML, with the positions of what is wrong.

use std::path::Path;

use serde::de::DeserializeOwned;

use super::ConfigError;

/// The language a configuration file is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    /// YAML 1.2, in a file ending in `.yaml` or `.yml`.
    Yaml,
    /// TOML 1.1, in a file ending in `.toml`.
    Toml,
}

impl Syntax {
    /// The language of the file at `path`, by its extension: TOML for `.toml`, YAML for anything else.
    #[must_use]
    pub fn of(path: &Path) -> Self {
        if path.extension().is_some_and(|extension| extension == "toml") {
            Self::Toml
        } else {
            Self::Yaml
        }
    }
}

/// Reads `text` into `T`; an empty file, or one of comments only, gives `None`.
pub(super) fn parse<T: DeserializeOwned>(text: &str, file: &str, syntax: Syntax) -> Result<Option<T>, ConfigError> {
    match syntax {
        Syntax::Yaml => serde_saphyr::from_str_with_options(text, yaml_options()).map_err(|e| yaml_error(file, &e)),
        Syntax::Toml => toml::from_str(text).map(Some).map_err(|e| toml_error(file, text, &e)),
    }
}

/// The YAML parser's options: messages without the snippet of the file, which an issue line has no room for.
fn yaml_options() -> serde_saphyr::Options {
    serde_saphyr::options! { with_snippet: false }
}

/// The error of the YAML parser as one line: the line of the file the problem is on, and what is wrong.
fn yaml_error(file: &str, e: &serde_saphyr::Error) -> ConfigError {
    let message = serde_saphyr::MessageFormatter::format_message(&serde_saphyr::UserMessageFormatter, e);
    let line = e
        .location()
        .map_or(1, |location| usize::try_from(location.line()).unwrap_or(usize::MAX));
    ConfigError::Parse {
        file: file.to_owned(),
        line,
        message: message.split_whitespace().collect::<Vec<_>>().join(" "),
    }
}

/// The error of the TOML parser as one line: the line its span starts on, and what is wrong.
fn toml_error(file: &str, text: &str, e: &toml::de::Error) -> ConfigError {
    let line = e
        .span()
        .map_or(1, |span| text[..span.start.min(text.len())].matches('\n').count() + 1);
    let message = e.message().split_whitespace().collect::<Vec<_>>().join(" ");
    ConfigError::Parse {
        file: file.to_owned(),
        line,
        message,
    }
}
