//! The JSON schema of the configuration file, `jsonschema/markright.jsonschema.json`, generated from the types the
//! configuration is read into, so editors check the file exactly as markright does.

use std::fs;
use std::io;
use std::path::Path;

use markright::config::Config;
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
