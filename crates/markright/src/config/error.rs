//! Why a configuration could not be used.

use std::io;
use std::path::PathBuf;

use super::{TOML_FILE, YAML_FILE};

/// Why a configuration could not be used.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
	/// The file could not be read.
	#[error("reading {path}: {source}")]
	Read {
		/// The file.
		path: String,
		/// Why.
		source: io::Error,
	},
	/// The file is not a configuration: its syntax or its structure is wrong.
	#[error("{file}: line {line}: {message}")]
	Parse {
		/// The name of the file.
		file: String,
		/// The line of the file the problem is on.
		line: usize,
		/// What is wrong.
		message: String,
	},
	/// The root holds both configuration files, and it would be unclear which one counts.
	#[error("both {YAML_FILE} and {TOML_FILE} exist in {}; keep one", root.display())]
	Ambiguous {
		/// The root of the repository.
		root: PathBuf,
	},
	/// The file reads, but what it says is not valid.
	#[error("{file}: {message}")]
	Invalid {
		/// The name of the file.
		file: String,
		/// What is wrong.
		message: String,
	},
}
