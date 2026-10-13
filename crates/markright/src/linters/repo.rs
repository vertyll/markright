//! The repository the linters look at, and the context each linter runs in.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::sync::{Mutex, OnceLock, PoisonError};

use super::Settings;
use super::targets::{parent, targets};
use crate::fs::Files;
use crate::markdown::Document;

/// What a linter gets besides the document: the repository, the settings of the linters and the linter path
/// exclusions.
pub(crate) struct Context<'a> {
	/// The repository being linted.
	pub(crate) repo: &'a Repo<'a>,
	/// The settings of the linters, of which the linter reads its own.
	pub(crate) settings: &'a Settings,
	/// Whether the linter exclusions cover the document at a path.
	pub(crate) excluded: &'a (dyn Fn(&str) -> bool + Sync),
}

/// The repository being linted: its files, which links may point at, and every document, parsed after formatting.
/// Linters may use it from many threads at once.
pub(crate) struct Repo<'a> {
	files: &'a dyn Files,
	/// Every document, by path.
	pub(crate) docs: BTreeMap<String, Document>,
	/// Every directory that holds a document, the root included.
	dirs: OnceLock<HashSet<String>>,
	/// Whether each path looked up on the files exists.
	looked_up: Mutex<HashMap<String, bool>>,
}

impl<'a> Repo<'a> {
	/// A repository of `files` whose documents are `docs`.
	#[must_use]
	pub(crate) fn new(files: &'a dyn Files, docs: BTreeMap<String, Document>) -> Self {
		Self {
			files,
			docs,
			dirs: OnceLock::new(),
			looked_up: Mutex::new(HashMap::new()),
		}
	}

	/// Whether `path`, in the repository, is a file or a directory. The documents and the directories that hold them
	/// exist without asking the files; any other path is looked up once. A repo lives for one pass, which sees the
	/// repository as it was when the pass read it, so nothing is remembered from one run to the next.
	///
	/// # Errors
	///
	/// Returns why the files could not say.
	pub(crate) fn exists(&self, path: &str) -> io::Result<bool> {
		if self.docs.contains_key(path) || self.dirs.get_or_init(|| self.index_dirs()).contains(path) {
			return Ok(true);
		}
		let known = self
			.looked_up
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.get(path)
			.copied();
		if let Some(known) = known {
			return Ok(known);
		}
		let exists = self.files.exists(path)?;
		self.looked_up
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.insert(path.to_owned(), exists);
		Ok(exists)
	}

	fn index_dirs(&self) -> HashSet<String> {
		let mut dirs = HashSet::from([".".to_owned()]);
		for path in self.docs.keys() {
			let mut dir = parent(path);
			while dirs.insert(dir.to_owned()) {
				dir = parent(dir);
			}
		}
		dirs
	}

	/// The files the document at `path` links, resolved.
	pub(super) fn linked(&self, path: &str) -> HashSet<String> {
		self.docs
			.get(path)
			.map(|document| {
				targets(path, document)
					.into_iter()
					.filter_map(|target| target.file)
					.collect()
			})
			.unwrap_or_default()
	}
}
