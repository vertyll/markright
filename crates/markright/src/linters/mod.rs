//! The registry of linters: rules that report issues in a document or in the whole repository.

mod alerts;
mod anchors;
mod blockquotes;
mod fence_language;
mod fences;
mod heading_increment;
mod image_alt;
mod index;
mod line_length;
mod links;
mod readme_links;
pub mod settings;
mod table_columns;
mod unused_definitions;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::io;
use std::sync::{Mutex, OnceLock, PoisonError};

use percent_encoding::percent_decode_str;

use crate::Error;
use crate::fs::Files;
use crate::issue::Finding;
use crate::markdown::{Document, is_external};
pub use settings::Settings;

/// What a linter is called and what it enforces, as the commands, the reports and the documentation show it.
#[derive(Debug, Clone, Copy)]
pub struct About {
	/// The name the configuration and the issues use.
	pub name: &'static str,
	/// One line of Markdown whose only markup is code spans; [`crate::markdown::plain`] gives it as text.
	pub description: &'static str,
	/// The release that added the linter.
	pub since: &'static str,
	/// A document the linter reports, and the same document fixed.
	pub example: Example,
}

/// A linter that looks at one document at a time.
pub trait DocumentLinter: Sync {
	/// What the linter is called and what it enforces.
	fn about(&self) -> About;

	/// What the linter finds in the document at `path`.
	///
	/// # Errors
	///
	/// Returns why the repository could not answer what the linter asked, such as whether a target exists.
	fn check(&self, context: &Context<'_>, path: &str, document: &Document) -> Result<Vec<Finding>, Error>;
}

/// A linter that looks at the whole repository once, after every document is parsed.
pub trait RepositoryLinter: Sync {
	/// What the linter is called and what it enforces.
	fn about(&self) -> About;

	/// What the linter finds in the repository.
	fn check(&self, context: &Context<'_>) -> Vec<Finding>;
}

/// A linter of the registry: of one document at a time, or of the whole repository.
#[derive(Clone, Copy)]
pub enum Linter {
	/// Runs once per document.
	Document(&'static dyn DocumentLinter),
	/// Runs once for the whole repository.
	Repository(&'static dyn RepositoryLinter),
}

impl Linter {
	/// What the linter is called and what it enforces.
	#[must_use]
	pub fn about(self) -> About {
		match self {
			Self::Document(linter) => linter.about(),
			Self::Repository(linter) => linter.about(),
		}
	}
}

impl fmt::Debug for Linter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Linter").field(&self.about().name).finish()
	}
}

/// A linter at work: `bad` is a document it reports, `good` the same document fixed. `path` is where the document
/// lives, and `files` are the other files of the repository the linter looks at.
#[derive(Debug, Clone, Copy)]
pub struct Example {
	/// A document the linter reports.
	pub bad: &'static str,
	/// The document fixed.
	pub good: &'static str,
	/// Where the document lives.
	pub path: &'static str,
	/// The other files of the repository, by path.
	pub files: &'static [(&'static str, &'static str)],
}

impl Example {
	const fn new(bad: &'static str, good: &'static str) -> Self {
		Self {
			bad,
			good,
			path: "doc.md",
			files: &[],
		}
	}

	const fn with_files(self, files: &'static [(&'static str, &'static str)]) -> Self {
		Self { files, ..self }
	}

	const fn at(self, path: &'static str) -> Self {
		Self { path, ..self }
	}
}

/// What a linter gets besides the document: the repository, the settings of the linters and the linter path
/// exclusions.
pub struct Context<'a> {
	/// The repository being linted.
	pub repo: &'a Repo<'a>,
	/// The settings of the linters, of which the linter reads its own.
	pub settings: &'a Settings,
	/// Whether the linter exclusions cover the document at a path.
	pub excluded: &'a (dyn Fn(&str) -> bool + Sync),
}

/// The repository being linted: its files, which links may point at, and every document, parsed after formatting.
/// Linters may use it from many threads at once.
pub struct Repo<'a> {
	files: &'a dyn Files,
	/// Every document, by path.
	pub docs: BTreeMap<String, Document>,
	/// Every directory that holds a document, the root included.
	dirs: OnceLock<HashSet<String>>,
	/// Whether each path looked up on the files exists.
	looked_up: Mutex<HashMap<String, bool>>,
}

impl<'a> Repo<'a> {
	/// A repository of `files` whose documents are `docs`.
	#[must_use]
	pub fn new(files: &'a dyn Files, docs: BTreeMap<String, Document>) -> Self {
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
	pub fn exists(&self, path: &str) -> io::Result<bool> {
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
	fn linked(&self, path: &str) -> HashSet<String> {
		self.docs
			.get(path)
			.map(|doc| targets(path, doc).into_iter().filter_map(|t| t.file).collect())
			.unwrap_or_default()
	}
}

/// The index the index linter checks and the readme-links linter expects the README to link, unless the
/// configuration names others.
const INDEX_DOCUMENT: &str = "CONTENTS.md";
const EXAMPLE_GUIDE: (&str, &str) = ("docs/guide.md", "# Guide\n\n## Install\n");

/// Every linter, in the order they run.
pub static ALL: &[Linter] = &[
	Linter::Document(&links::Links),
	Linter::Document(&anchors::Anchors),
	Linter::Document(&unused_definitions::UnusedDefinitions),
	Linter::Document(&alerts::Alerts),
	Linter::Document(&blockquotes::Blockquotes),
	Linter::Document(&fences::Fences),
	Linter::Document(&fence_language::FenceLanguage),
	Linter::Document(&heading_increment::HeadingIncrement),
	Linter::Document(&image_alt::ImageAlt),
	Linter::Document(&table_columns::TableColumns),
	Linter::Document(&line_length::LineLength),
	Linter::Repository(&index::Index),
	Linter::Repository(&readme_links::ReadmeLinks),
];

/// The linter called `name`.
#[must_use]
pub fn find(name: &str) -> Option<Linter> {
	ALL.iter().copied().find(|l| l.about().name == name)
}

/// A link of a document to a place in the repository: the file it resolves to, unless it points into the document
/// itself, and the fragment it names, if any.
struct Target<'a> {
	line: usize,
	raw: &'a str,
	file: Option<String>,
	anchor: Option<String>,
}

impl Target<'_> {
	/// The document the target's fragment is looked up in: the file it names, or the linking document itself.
	fn document<'p>(&'p self, from: &'p str) -> &'p str {
		self.file.as_deref().unwrap_or(from)
	}
}

/// The links of the document at `path` that stay in the repository, resolved against its directory.
fn targets<'d>(path: &str, doc: &'d Document) -> Vec<Target<'d>> {
	doc.links
		.iter()
		.filter(|l| !l.target.is_empty() && !is_external(&l.target))
		.map(|l| {
			let (file, anchor) = l
				.target
				.split_once('#')
				.map_or((l.target.as_str(), None), |(f, a)| (f, Some(a)));
			let file = file.split_once('?').map_or(file, |(f, _)| f);
			Target {
				line: l.line,
				raw: &l.target,
				file: (!file.is_empty()).then(|| join(parent(path), &unescape(file))),
				anchor: anchor.map(unescape),
			}
		})
		.collect()
}

/// Decodes the percent escapes of a link target, so `my%20file.md` names `my file.md`; a target whose escapes do not
/// make UTF-8 stays as written.
fn unescape(s: &str) -> String {
	percent_decode_str(s)
		.decode_utf8()
		.map_or_else(|_| s.to_owned(), Cow::into_owned)
}

/// The directory of a slash-separated path, `.` for one at the root.
fn parent(path: &str) -> &str {
	path.rsplit_once('/')
		.map_or(".", |(dir, _)| if dir.is_empty() { "/" } else { dir })
}

/// Joins `relative` to `dir` and resolves its `.` and `..` elements; a path that climbs above the root keeps its
/// leading `..`.
fn join(dir: &str, relative: &str) -> String {
	let mut parts: Vec<&str> = Vec::new();
	for part in dir.split('/').chain(relative.split('/')) {
		match part {
			"" | "." => {}
			".." if parts.last().is_some_and(|&last| last != "..") => {
				parts.pop();
			}
			_ => parts.push(part),
		}
	}
	if parts.is_empty() {
		".".to_owned()
	} else {
		parts.join("/")
	}
}

/// Whether a resolved path climbs out of the repository.
fn escapes(resolved: &str) -> bool {
	resolved == ".." || resolved.starts_with("../")
}

#[cfg(test)]
mod tests;
