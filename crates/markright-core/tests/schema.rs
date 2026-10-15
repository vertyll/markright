//! Holds the examples of the configuration guide to the JSON schema of the configuration file, which `cargo xtask
//! schema` generates from the configuration's types: the "Every key" example must be a valid configuration that sets
//! every key the schema defines and no other, and its TOML form must say exactly what its YAML form says.

use std::collections::BTreeSet;
use std::path::Path;

use markright_core::config::{self, Config, Syntax};
use serde_json::Value;

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the crate is two levels down")
}

fn schema() -> Value {
    let raw = std::fs::read_to_string(repository().join("jsonschema/markright.jsonschema.json")).expect("the schema");
    serde_json::from_str(&raw).expect("the schema is JSON")
}

/// The block of the configuration guide that starts right after `start`.
fn block(start: &str) -> String {
    let guide = std::fs::read_to_string(repository().join("docs/configuration.md")).expect("the configuration guide");
    let (_, rest) = guide
        .split_once(start)
        .unwrap_or_else(|| panic!("the guide has no {start:?}"));
    rest.split_once("\n```\n").expect("the block is closed").0.to_owned()
}

fn yaml_example() -> String {
    block("Every key:\n\n```yaml\n")
}

fn toml_example() -> String {
    block("The same in TOML:\n\n```toml\n")
}

/// `node` with a reference to a definition, or a choice between one and nothing, replaced by the definition.
fn resolve<'a>(root: &'a Value, node: &'a Value) -> &'a Value {
    let node = node["anyOf"]
        .as_array()
        .and_then(|choices| choices.iter().find(|c| c["type"] != "null"))
        .unwrap_or(node);
    match node["$ref"].as_str().and_then(|r| r.strip_prefix("#/$defs/")) {
        Some(name) => &root["$defs"][name],
        None => node,
    }
}

/// Every key the schema defines under `node`, as a dotted path; `[]` stands for the items of a list.
fn schema_keys(root: &Value, node: &Value, path: &str, out: &mut BTreeSet<String>) {
    let node = resolve(root, node);
    let join = |key: &str| {
        if path.is_empty() {
            key.to_owned()
        } else {
            format!("{path}.{key}")
        }
    };
    if let Some(properties) = node["properties"].as_object() {
        for (key, child) in properties {
            schema_keys(root, child, &join(key), out);
        }
    } else if resolve(root, &node["items"])["properties"].is_object() {
        schema_keys(root, &node["items"], &format!("{path}[]"), out);
    } else {
        out.insert(path.to_owned());
    }
}

/// Every key `value` sets, as `schema_keys` names them.
fn value_keys(value: &Value, path: &str, out: &mut BTreeSet<String>) {
    let join = |key: &str| {
        if path.is_empty() {
            key.to_owned()
        } else {
            format!("{path}.{key}")
        }
    };
    match value {
        Value::Object(map) if !map.is_empty() => {
            for (key, child) in map {
                value_keys(child, &join(key), out);
            }
        }
        Value::Array(items) if items.first().is_some_and(Value::is_object) => {
            for item in items {
                value_keys(item, &format!("{path}[]"), out);
            }
        }
        _ => {
            out.insert(path.to_owned());
        }
    }
}

#[test]
fn the_example_shows_every_key() {
    let root = schema();
    Config::parse(&yaml_example(), config::YAML_FILE, Syntax::Yaml).expect("the example is a valid configuration");
    let example: Value = serde_saphyr::from_str(&yaml_example()).expect("the example is YAML");
    let (mut shown, mut defined) = (BTreeSet::new(), BTreeSet::new());
    value_keys(&example, "", &mut shown);
    schema_keys(&root, &root, "", &mut defined);
    assert_eq!(
        shown, defined,
        "the example sets every key the schema defines, and no other"
    );
}

#[test]
fn the_toml_example_says_what_the_yaml_example_says() {
    let yaml = Config::parse(&yaml_example(), config::YAML_FILE, Syntax::Yaml).expect("the YAML example");
    let toml = Config::parse(&toml_example(), config::TOML_FILE, Syntax::Toml).expect("the TOML example");
    assert_eq!(toml, yaml);
}
