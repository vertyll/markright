//! The licenses of the crates a build includes, which go with what is distributed: the archives of a release and the
//! playground of the documentation site. `cargo tree` says which crates a package uses on a target, with the features
//! that build turns on, `cargo metadata` where each crate is, and each crate's package holds its license files.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// The names a license file starts with, in upper case.
const LICENSE_FILES: [&str; 5] = ["LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE"];

/// What `cargo metadata` says of the packages it knows.
#[derive(Deserialize)]
struct Metadata {
	packages: Vec<Package>,
}

/// A package as `cargo metadata` describes it.
#[derive(Deserialize)]
struct Package {
	name: String,
	version: String,
	/// An SPDX expression, such as `MIT OR Apache-2.0`.
	license: Option<String>,
	/// A license file the manifest names, relative to the package.
	license_file: Option<PathBuf>,
	repository: Option<String>,
	manifest_path: PathBuf,
}

/// Prints the license of every crate that `package` of the workspace at `root` includes when built for `target`.
pub(crate) fn print(root: &Path, package: &str, target: &str) -> io::Result<()> {
	let tree = cargo(
		root,
		&[
			"tree",
			"--locked",
			"--package",
			package,
			"--target",
			target,
			"--edges",
			"normal",
			"--prefix",
			"none",
		],
	)?;
	let tree = String::from_utf8(tree).map_err(io::Error::other)?;
	let metadata: Metadata = serde_json::from_slice(&cargo(root, &["metadata", "--format-version", "1", "--locked"])?)
		.map_err(io::Error::other)?;
	print!("{}", notice(&metadata, &included(&tree), package, target)?);
	Ok(())
}

/// What `cargo` with `args`, run in `root`, prints, or what it says went wrong.
fn cargo(root: &Path, args: &[&str]) -> io::Result<Vec<u8>> {
	let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
	let output = Command::new(cargo).current_dir(root).args(args).output()?;
	if output.status.success() {
		Ok(output.stdout)
	} else {
		Err(io::Error::other(String::from_utf8_lossy(&output.stderr).into_owned()))
	}
}

/// The crates `cargo tree` lists, by name and version, without those of the workspace, which it lists with their
/// path.
fn included(tree: &str) -> BTreeSet<(&str, &str)> {
	tree.lines()
		.filter_map(|line| {
			let mut words = line.split_whitespace();
			let name = words.next()?;
			let version = words.next()?.strip_prefix('v')?;
			let local = words
				.next()
				.is_some_and(|word| word.starts_with('(') && word.contains('/'));
			(!local).then_some((name, version))
		})
		.collect()
}

/// The text that lists every crate of `crates`, with its license files, which `metadata` says where to find.
fn notice(metadata: &Metadata, crates: &BTreeSet<(&str, &str)>, package: &str, target: &str) -> io::Result<String> {
	let mut text = format!(
		"{package}, built for {target}, includes the crates below. Each is listed with its license and the license \
		 files it is distributed with.\n"
	);
	for &(name, version) in crates {
		let krate = metadata
			.packages
			.iter()
			.find(|package| package.name == name && package.version == version)
			.ok_or_else(|| io::Error::other(format!("cargo metadata does not know {name} {version}")))?;
		let license = krate.license.as_deref();
		let _ = write!(text, "\n{}\n{name} {version}\n", "=".repeat(80));
		let _ = writeln!(text, "License: {}", license.unwrap_or("see the files below"));
		if let Some(repository) = &krate.repository {
			let _ = writeln!(text, "Source: {repository}");
		}
		let files = license_files(krate)?;
		if files.is_empty() {
			// A few packages leave their license files out; the standard texts of their licenses stand in.
			text.push_str("\nThe package holds no license file. The text of each license it names:\n\n");
			for id in spdx_ids(license.unwrap_or_default()) {
				let _ = writeln!(text, "{id}: https://spdx.org/licenses/{id}.html");
			}
		}
		for (file, content) in files {
			let _ = write!(text, "\n--- {file} ---\n\n{}\n", content.trim_end());
		}
	}
	Ok(text)
}

/// The licenses an SPDX expression such as `MIT OR Apache-2.0` names, each once.
fn spdx_ids(expression: &str) -> BTreeSet<&str> {
	expression
		.split(|c: char| c.is_whitespace() || "()/".contains(c))
		.filter(|word| !word.is_empty() && !["AND", "OR", "WITH"].contains(word))
		.collect()
}

/// The license files in a crate's package, by name, with their text.
fn license_files(krate: &Package) -> io::Result<BTreeMap<String, String>> {
	let directory = krate
		.manifest_path
		.parent()
		.ok_or_else(|| io::Error::other("a manifest is in a directory"))?;
	let mut files = BTreeMap::new();
	for entry in fs::read_dir(directory)? {
		let entry = entry?;
		let name = entry.file_name().to_string_lossy().into_owned();
		let upper = name.to_uppercase();
		if entry.file_type()?.is_file() && LICENSE_FILES.iter().any(|prefix| upper.starts_with(prefix)) {
			files.insert(name, fs::read_to_string(entry.path())?);
		}
	}
	if let Some(file) = &krate.license_file {
		files.insert(file.display().to_string(), fs::read_to_string(directory.join(file))?);
	}
	Ok(files)
}

#[cfg(test)]
mod tests {
	use super::{included, spdx_ids};

	#[test]
	fn the_crates_of_a_tree_without_the_workspace() {
		let tree = "markright-cli v0.1.0 (/repo/crates/markright-cli)\nclap v4.6.7\nserde_derive v1.0.229 (proc-macro)\n\
		            clap v4.6.7 (*)\nmarkright v0.1.0 (/repo/crates/markright)\n";
		let crates: Vec<_> = included(tree).into_iter().collect();
		assert_eq!(crates, [("clap", "4.6.7"), ("serde_derive", "1.0.229")]);
	}

	#[test]
	fn the_licenses_an_expression_names() {
		let ids: Vec<_> = spdx_ids("(MIT OR Apache-2.0) AND Unicode-3.0").into_iter().collect();
		assert_eq!(ids, ["Apache-2.0", "MIT", "Unicode-3.0"]);
		let ids: Vec<_> = spdx_ids("Apache-2.0 WITH LLVM-exception OR MIT/Apache-2.0")
			.into_iter()
			.collect();
		assert_eq!(ids, ["Apache-2.0", "LLVM-exception", "MIT"]);
	}
}
