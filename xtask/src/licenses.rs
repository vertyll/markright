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

/// The standard texts of the licenses a package may name without holding a license file, by SPDX identifier.
const TEXTS: [(&str, &str); 3] = [
    ("Apache-2.0", include_str!("../licenses/Apache-2.0.txt")),
    ("MIT", include_str!("../licenses/MIT.txt")),
    ("Zlib", include_str!("../licenses/Zlib.txt")),
];

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
    /// Who wrote it, as its manifest names them: the holders of its copyright.
    #[serde(default)]
    authors: Vec<String>,
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
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(rustc).current_dir(root).arg("-vV").output()?;
    let version = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    print!(
        "{}{}",
        notice(&metadata, &included(&tree), package, target)?,
        standard_library(&version)
    );
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
            text.push_str(&standard_texts(krate));
        }
        for (file, content) in files {
            let _ = write!(text, "\n--- {file} ---\n\n{}\n", content.trim_end());
        }
    }
    Ok(text)
}

/// For a package that leaves its license files out, its copyright holders and the standard text of each license it
/// names, or a link to the text of one this file does not hold.
fn standard_texts(krate: &Package) -> String {
    let authors = if krate.authors.is_empty() {
        "the authors of the package".to_owned()
    } else {
        krate.authors.join(", ")
    };
    let mut text = format!(
        "\nThe package holds no license file; the standard text of each license it names follows.\nCopyright (c) \
		 {authors}\n"
    );
    text.push_str(&license_texts(krate.license.as_deref().unwrap_or_default()));
    text
}

/// The standard text of each license `expression` names, or a link to the text of one this file does not hold.
fn license_texts(expression: &str) -> String {
    let mut text = String::new();
    for id in spdx_ids(expression) {
        match TEXTS.iter().find(|&&(known, _)| known == id) {
            Some((_, standard)) => {
                let _ = write!(text, "\n--- {id} ---\n\n{}\n", standard.trim_end());
            }
            None => {
                let _ = writeln!(text, "\n--- {id} ---\n\nhttps://spdx.org/licenses/{id}.html");
            }
        }
    }
    text
}

/// The entry of the Rust standard library, which every Rust program includes though no `cargo tree` lists it, for the
/// compiler that `rustc -vV` describes as `version`. The crates it is built from are listed, with their licenses, in
/// the COPYRIGHT file of that compiler's source.
fn standard_library(version: &str) -> String {
    let field = |name: &str| {
        version
            .lines()
            .find_map(|line| line.strip_prefix(name)?.strip_prefix(": "))
            .unwrap_or("unknown")
    };
    let (release, commit) = (field("release"), field("commit-hash"));
    let mut text = format!(
        "\n{}\nRust standard library {release}\nLicense: MIT OR Apache-2.0\nSource: https://github.com/rust-lang/rust\n\nEvery \
		 Rust program includes it. Copyright (c) The Rust Project Developers. The crates it is built from, with their \
		 licenses, are listed in https://github.com/rust-lang/rust/blob/{commit}/COPYRIGHT\n",
        "=".repeat(80)
    );
    text.push_str(&license_texts("MIT OR Apache-2.0"));
    text
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
    use std::path::PathBuf;

    use super::{Package, included, spdx_ids, standard_library, standard_texts};

    #[test]
    fn the_standard_library_is_listed_for_the_compiler_in_use() {
        let text = standard_library("rustc 1.99.0 (b940084d7 2026-09-28)\ncommit-hash: b940084d7\nrelease: 1.99.0\n");
        assert!(
            text.contains("Rust standard library 1.99.0\nLicense: MIT OR Apache-2.0"),
            "{text}"
        );
        assert!(text.contains("rust-lang/rust/blob/b940084d7/COPYRIGHT"), "{text}");
        assert!(
            text.contains("--- Apache-2.0 ---") && text.contains("--- MIT ---"),
            "{text}"
        );
    }

    #[test]
    fn a_package_without_license_files_gets_its_authors_and_the_standard_texts() {
        let krate = Package {
            name: "x".to_owned(),
            version: "1.0.0".to_owned(),
            license: Some("Zlib OR MIT OR Unknown-1.0".to_owned()),
            license_file: None,
            authors: vec!["A <a@example.com>".to_owned()],
            repository: None,
            manifest_path: PathBuf::new(),
        };
        let text = standard_texts(&krate);
        assert!(text.contains("Copyright (c) A <a@example.com>"), "{text}");
        assert!(
            text.contains("--- MIT ---\n\nMIT License\n\nPermission is hereby granted"),
            "{text}"
        );
        assert!(text.contains("--- Zlib ---\n\nzlib License"), "{text}");
        assert!(
            text.contains("--- Unknown-1.0 ---\n\nhttps://spdx.org/licenses/Unknown-1.0.html"),
            "{text}"
        );
    }

    #[test]
    fn the_crates_of_a_tree_without_the_workspace() {
        let tree = "markright v0.1.0 (/repo/crates/markright)\nclap v4.6.7\nserde_derive v1.0.229 (proc-macro)\n\
		            clap v4.6.7 (*)\nmarkright-core v0.1.0 (/repo/crates/markright-core)\n";
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
