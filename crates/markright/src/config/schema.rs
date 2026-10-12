//! The parts of the configuration's JSON schema that its types alone cannot say: the version it must declare, and the
//! names the registries hold.

use schemars::{Schema, SchemaGenerator, json_schema};

use super::VERSION;
use crate::{formatters, linters};

pub(super) fn version(_: &mut SchemaGenerator) -> Schema {
	json_schema!({ "type": "string", "const": VERSION })
}

pub(super) fn linter_names_list() -> Vec<&'static str> {
	linters::ALL.iter().map(|l| l.about().name).collect()
}

pub(super) fn formatter_names_list() -> Vec<&'static str> {
	formatters::ALL.iter().map(|f| f.about().name).collect()
}

/// A list of linter names, each from the registry, each once.
pub(super) fn linter_names(_: &mut SchemaGenerator) -> Schema {
	names(&linter_names_list())
}

/// A list of formatter names, each from the registry, each once.
pub(super) fn formatter_names(_: &mut SchemaGenerator) -> Schema {
	names(&formatter_names_list())
}

fn names(names: &[&str]) -> Schema {
	json_schema!({ "type": "array", "items": { "type": "string", "enum": names }, "uniqueItems": true })
}
