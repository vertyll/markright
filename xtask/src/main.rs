//! The project's own tasks, run with `cargo xtask <task>` from anywhere in the repository.

mod licenses;
mod schema;
mod site;
mod tables;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "Usage: cargo xtask <task>

Tasks:
  schema          write jsonschema/markright.jsonschema.json from the types of the configuration
  schema --check  fail when the schema is out of date
  tables          write the tables of docs/rules.md and docs/configuration.md from the registries and the schema
  tables --check  fail when those tables are out of date
  site            write the pages of the documentation site into site/content
  site check <dir> <base>
                  fail when a link of the site built into <dir>, served at <base>, leads nowhere
  licenses <package> <target>
                  print the licenses of the crates <package> includes when built for <target>
";

fn main() -> ExitCode {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let args: Vec<&str> = args.iter().map(String::as_str).collect();
	let result = match args.as_slice() {
		["schema"] => schema::write(&root()),
		["schema", "--check"] => schema::check(&root()),
		["tables"] => tables::write(&root()),
		["tables", "--check"] => tables::check(&root()),
		["site"] => site::generate(&root()),
		["site", "check", public, base] => site::check(Path::new(public), base),
		["licenses", package, target] => licenses::print(&root(), package, target),
		_ => {
			eprint!("{USAGE}");
			return ExitCode::from(2);
		}
	};
	match result {
		Ok(()) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("xtask: {e}");
			ExitCode::FAILURE
		}
	}
}

/// The root of the repository: the directory above this crate.
fn root() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.expect("xtask lives in the repository")
		.to_owned()
}
