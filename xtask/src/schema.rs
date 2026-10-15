//! The JSON schema of the configuration file, `jsonschema/markright.jsonschema.json`, generated from the types the
//! configuration is read into, so editors check the file exactly as markright does.

use std::fs;
use std::io;
use std::path::Path;

use markright_core::config::Config;
use markright_core::{formatters, linters};
use serde_json::Value;

/// Where the schema lives in the repository.
pub(crate) const PATH: &str = "jsonschema/markright.jsonschema.json";
/// Where the schema is published, as its `$id` says.
const ID: &str = "https://raw.githubusercontent.com/vertyll/markright/main/jsonschema/markright.jsonschema.json";

/// The schema, as the file holds it.
pub(crate) fn generate() -> io::Result<String> {
    let schema = serde_json::to_value(schemars::schema_for!(Config)).map_err(io::Error::other)?;
    let Value::Object(fields) = schema else {
        return Err(io::Error::other("the schema is not an object"));
    };
    let mut with_id = serde_json::Map::new();
    for (key, value) in fields {
        let after_schema = key == "$schema";
        with_id.insert(key, value);
        if after_schema {
            with_id.insert("$id".to_owned(), Value::String(ID.to_owned()));
        }
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(with_id)).map_err(io::Error::other)?;
    text.push('\n');
    Ok(text)
}

/// Writes the schema.
pub(crate) fn write(root: &Path) -> io::Result<()> {
    let schema = generate()?;
    if fs::read_to_string(root.join(PATH)).ok().as_deref() != Some(schema.as_str()) {
        fs::write(root.join(PATH), schema)?;
        println!("wrote {PATH}");
    }
    Ok(())
}

/// A setting of a formatter or a linter, as the schema describes it.
pub(crate) struct Setting {
    /// `formatters` or `linters`.
    pub(crate) section: &'static str,
    /// The rule it belongs to.
    pub(crate) rule: &'static str,
    /// Its key under the rule.
    pub(crate) key: String,
    /// What it means, as a sentence on one line.
    pub(crate) description: String,
    /// Its default.
    pub(crate) default: Value,
}

/// Every setting the schema in the repository defines: the formatters' first, then the linters', each section in the
/// order of its registry and each rule's in the order of the schema.
pub(crate) fn settings(root: &Path) -> io::Result<Vec<Setting>> {
    let raw = fs::read_to_string(root.join(PATH))?;
    let schema: Value = serde_json::from_str(&raw).map_err(io::Error::other)?;
    let resolve = |node: &'_ Value| -> Value {
        let name = node["$ref"]
            .as_str()
            .and_then(|reference| reference.strip_prefix("#/$defs/"));
        name.map_or_else(|| node.clone(), |name| schema["$defs"][name].clone())
    };
    let sections = [("formatters", formatters::names()), ("linters", linters::names())];
    let mut settings = Vec::new();
    for (section, names) in sections {
        let rules = resolve(&resolve(&schema["properties"][section])["properties"]["settings"]);
        for rule in names {
            let Some(keys) = resolve(&rules["properties"][rule])["properties"].as_object().cloned() else {
                continue;
            };
            for (key, property) in keys {
                settings.push(Setting {
                    section,
                    rule,
                    key,
                    // A description written over several lines of a doc comment reads as one sentence.
                    description: property["description"]
                        .as_str()
                        .unwrap_or_default()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                    default: property["default"].clone(),
                });
            }
        }
    }
    Ok(settings)
}

/// Fails when the schema is not what the types make of it.
pub(crate) fn check(root: &Path) -> io::Result<()> {
    if fs::read_to_string(root.join(PATH))? == generate()? {
        Ok(())
    } else {
        Err(io::Error::other(format!("out of date: {PATH}; run cargo xtask schema")))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_schema_is_current() {
        super::check(&crate::root()).unwrap();
    }
}
