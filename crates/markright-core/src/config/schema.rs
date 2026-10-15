//! The parts of the configuration's JSON schema that its types alone cannot say: the version it must declare, and the
//! names the registries hold.

use schemars::{Schema, SchemaGenerator, json_schema};

use super::VERSION;
use crate::{formatters, linters};

/// The version, which must be the one this release reads.
pub(super) fn version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({ "type": "string", "const": VERSION })
}

/// A list of linter names, each from the registry, each once.
pub(super) fn linter_names(_: &mut SchemaGenerator) -> Schema {
    names(&linters::names())
}

/// A list of formatter names, each from the registry, each once.
pub(super) fn formatter_names(_: &mut SchemaGenerator) -> Schema {
    names(&formatters::names())
}

/// A list of `names`, each once.
fn names(names: &[&str]) -> Schema {
    json_schema!({ "type": "array", "items": { "type": "string", "enum": names }, "uniqueItems": true })
}
