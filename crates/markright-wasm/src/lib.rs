//! Checks one document the way `markright check` and `markright fmt` would, for the playground on the documentation site.
//! It runs the same formatters and linters as the command, in memory. Compiled to WebAssembly, it exports
//! `check(markdown, configuration, syntax)` to the page, which returns an [`Outcome`] as an object, or throws why the
//! document could not be checked.

use std::collections::BTreeMap;

use markright_core::config::{self, Config, ConfigError};
use markright_core::runner::{self, Mode};
use serde::Serialize;

/// The path the playground's document has: the only file of its repository, so a link to any other file is broken.
const DOCUMENT: &str = "doc.md";

/// One issue in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    /// The 1-based line.
    pub line: usize,
    /// The formatter or linter that found it.
    pub source: &'static str,
    /// What is wrong.
    pub message: String,
}

/// What the playground shows: the issues, the report `markright check` prints for them, and the document as
/// `markright fmt` leaves it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outcome {
    /// The issues, sorted by line.
    pub issues: Vec<Issue>,
    /// The text report.
    pub report: String,
    /// The formatted document.
    pub formatted: String,
}

/// Why the document could not be checked.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    /// The configuration is not valid.
    #[error("configuration: {0}")]
    Configuration(#[from] ConfigError),
    /// The syntax of the configuration is neither `yaml` nor `toml`.
    #[error("configuration: no syntax {0:?}; use yaml or toml")]
    Syntax(String),
    /// The check failed.
    #[error(transparent)]
    Run(#[from] markright_core::Error),
}

/// Checks `markdown` with `configuration`, the content of a `.markright.yaml` file when `syntax` is `yaml` or of a
/// `.markright.toml` file when it is `toml`; an empty one means the defaults.
///
/// # Errors
///
/// Returns why the document could not be checked: a configuration that is not valid, or an unknown syntax.
pub fn check(markdown: &str, configuration: &str, syntax: &str) -> Result<Outcome, CheckError> {
    let (file, syntax) = match syntax {
        "yaml" => (config::YAML_FILE, config::Syntax::Yaml),
        "toml" => (config::TOML_FILE, config::Syntax::Toml),
        other => return Err(CheckError::Syntax(other.to_owned())),
    };
    let config = if configuration.trim().is_empty() {
        Config::default()
    } else {
        Config::parse(configuration, file, syntax)?
    };
    let documents = BTreeMap::from([(DOCUMENT.to_owned(), markdown.to_owned())]);
    let checked = runner::check_documents(&documents, &config, Mode::Lint)?;
    Ok(Outcome {
        report: markright_core::output::text(&checked.issues),
        issues: checked
            .issues
            .into_iter()
            .map(|issue| Issue {
                line: issue.line,
                source: issue.source,
                message: issue.message,
            })
            .collect(),
        formatted: checked
            .fixed
            .get(DOCUMENT)
            .cloned()
            .unwrap_or_else(|| markdown.to_owned()),
    })
}

/// The export to the page: [`check`], with its outcome as a JavaScript object.
#[cfg(target_arch = "wasm32")]
pub mod exports {
    use wasm_bindgen::prelude::*;

    /// Checks `markdown` with `configuration`, written in `syntax`, `yaml` or `toml`, and returns the outcome as an
    /// object.
    ///
    /// # Errors
    ///
    /// Throws why the document could not be checked.
    #[wasm_bindgen]
    pub fn check(markdown: &str, configuration: &str, syntax: &str) -> Result<JsValue, JsError> {
        let outcome = super::check(markdown, configuration, syntax).map_err(|e| JsError::new(&e.to_string()))?;
        serde_wasm_bindgen::to_value(&outcome).map_err(|e| JsError::new(&e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckError, Issue, check};

    fn checked(markdown: &str, configuration: &str) -> super::Outcome {
        check(markdown, configuration, "yaml").unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn reports_what_run_reports() {
        let outcome = checked("# Title\n\nGo -> here.\n\n[Missing](#nowhere)\n", "");
        assert_eq!(
            outcome.report,
            "doc.md:3: ASCII arrow or missing space around an arrow (arrows)\ndoc.md:5: broken anchor: #nowhere (anchors)\n\
			 markright: 2 issue(s)\n"
        );
        let first = Issue {
            line: 3,
            source: "arrows",
            message: "ASCII arrow or missing space around an arrow".into(),
        };
        assert_eq!((outcome.issues.len(), &outcome.issues[0]), (2, &first));
        assert_eq!(outcome.formatted, "# Title\n\nGo → here.\n\n[Missing](#nowhere)\n");
    }

    #[test]
    fn a_clean_document() {
        let outcome = checked("# Title\n\nAll good.\n", "");
        assert_eq!(
            (outcome.report.as_str(), outcome.issues.len()),
            ("markright: 0 issues\n", 0)
        );
        assert_eq!(outcome.formatted, "# Title\n\nAll good.\n");
    }

    #[test]
    fn other_files_do_not_exist() {
        let outcome = checked("# Title\n\nSee [the guide](guide.md).\n", "");
        assert!(
            outcome.issues.len() == 1 && outcome.issues[0].source == "links",
            "{outcome:?}"
        );
    }

    #[test]
    fn the_configuration_chooses_the_rules() {
        let outcome = checked(
            "# Title\n\nGo -> here.\n",
            "version: \"1\"\nformatters:\n  disable: [arrows]\n",
        );
        assert!(
            outcome.issues.is_empty() && outcome.formatted == "# Title\n\nGo -> here.\n",
            "{outcome:?}"
        );
    }

    #[test]
    fn a_bad_configuration() {
        for configuration in [
            "version: \"2\"
",
            "version: \"1\"
linters:
  enable: [nope]
",
            "{",
        ] {
            let error = check(
                "Go -> here.
",
                configuration,
                "yaml",
            )
            .unwrap_err();
            assert!(matches!(error, CheckError::Configuration(_)), "{error}");
            assert!(
                error.to_string().starts_with("configuration: .markright.yaml: "),
                "{error}"
            );
        }
    }

    #[test]
    fn a_toml_configuration() {
        let outcome = check(
            "# Title\n\nGo -> here.\n",
            "version = \"1\"\n[formatters]\ndisable = [\"arrows\"]\n",
            "toml",
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert!(outcome.issues.is_empty(), "{outcome:?}");
        let error = check("x\n", "version = 2\n", "toml").unwrap_err();
        assert!(
            error.to_string().starts_with("configuration: .markright.toml: "),
            "{error}"
        );
    }

    #[test]
    fn an_unknown_syntax() {
        let error = check("x\n", "", "json").unwrap_err();
        assert!(matches!(error, CheckError::Syntax(_)), "{error}");
        assert_eq!(error.to_string(), "configuration: no syntax \"json\"; use yaml or toml");
    }
}
