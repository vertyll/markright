//! Makes one pass over a repository: it formats every document, lints the result and filters the issues through the
//! exclusions. [`Runner`] does it on disk and writes the result; [`check_documents`] does it in memory.

#[cfg(feature = "disk")]
mod disk;
mod parallel;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "disk")]
pub use disk::{Report, Runner};

use crate::Error;
use crate::config::{Config, ExclusionKey};
use crate::formatters;
use crate::fs::{Files, is_markdown};
use crate::issue::{Finding, Issue};
use crate::linters::{self, Context, Linter, Repo};
use crate::markdown::Document;

/// What a run does with the formatters' changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	/// Reports what the formatters would change and runs the linters.
	Lint,
	/// Keeps what the formatters change, and runs no linter.
	Format,
}

/// What a check found: the issues, sorted by path and line, the formatted content of every document the formatters
/// change, by path, and the warnings for watched exclusions that matched nothing.
#[derive(Debug, Default)]
pub struct Checked {
	/// The issues, sorted by path, line and message.
	pub issues: Vec<Issue>,
	/// The new content of each document the formatters change.
	pub fixed: BTreeMap<String, String>,
	/// A warning for each watched exclusion that matched nothing.
	pub unused_exclusions: Vec<String>,
}

/// Checks a repository held in memory: every file in `documents`, by path, is part of it, and the ones ending in
/// `.md` are the documents.
///
/// # Errors
///
/// Returns why the documents could not be checked.
pub fn check_documents(documents: &BTreeMap<String, String>, config: &Config, mode: Mode) -> Result<Checked, Error> {
	let paths: Vec<String> = documents.keys().filter(|p| is_markdown(p)).cloned().collect();
	check(documents, &paths, config, mode, &AtomicBool::new(false))
}

/// Formats the documents at `paths` in `files` and, in [`Mode::Lint`], lints the result. It reads `files` and never
/// writes to them, so it works the same on disk and in memory. It spreads the documents over the cores and stops
/// once `stop` is set.
pub(crate) fn check(
	files: &dyn Files,
	paths: &[String],
	config: &Config,
	mode: Mode,
	stop: &AtomicBool,
) -> Result<Checked, Error> {
	let mut pass = Pass {
		config,
		mode,
		issues: Vec::new(),
		used: HashSet::new(),
	};
	let (docs, fixed) = pass.format_all(files, paths, stop)?;
	let repo = Repo::new(files, docs);
	if mode == Mode::Lint {
		pass.lint(&repo, paths, stop)?;
	}
	let mut issues = pass.issues;
	issues.sort_by(|a, b| (&a.path, a.line, &a.message).cmp(&(&b.path, b.line, &b.message)));
	let unused = config
		.watched()
		.into_iter()
		.filter(|w| !pass.used.contains(&w.key))
		.map(|w| w.warning)
		.collect();
	Ok(Checked {
		issues,
		fixed,
		unused_exclusions: unused,
	})
}

/// Parsed documents, by path.
type Documents = BTreeMap<String, Document>;
/// File contents, by path.
type Contents = BTreeMap<String, String>;

/// What one pass over the documents finds.
struct Pass<'a> {
	config: &'a Config,
	mode: Mode,
	issues: Vec<Issue>,
	used: HashSet<ExclusionKey>,
}

/// What formatting one document gives: the document as the linters see it, its new content when a formatter changed
/// it, and, in [`Mode::Lint`], what each formatter would change.
struct Formatted {
	doc: Document,
	fixed: Option<String>,
	issues: Vec<Issue>,
}

impl Pass<'_> {
	/// Reads, parses and formats every document, spread over the cores, then records the results in the order of
	/// `paths`, so the issues and the first error are the same however the work was spread.
	fn format_all(
		&mut self,
		files: &dyn Files,
		paths: &[String],
		stop: &AtomicBool,
	) -> Result<(Documents, Contents), Error> {
		let skip: Vec<bool> = paths
			.iter()
			.map(|p| self.record(self.config.formatter_excluded(p)))
			.collect();
		let jobs: Vec<(&String, bool)> = paths.iter().zip(skip).collect();
		let results = parallel::map(&jobs, |&(path, skip)| {
			if stop.load(Ordering::Relaxed) {
				Err(Error::Stopped)
			} else {
				self.format(files, path, skip)
			}
		});
		if stop.load(Ordering::Relaxed) {
			return Err(Error::Stopped);
		}
		let (mut docs, mut fixed) = (BTreeMap::new(), BTreeMap::new());
		for (path, result) in paths.iter().zip(results) {
			let formatted = result?;
			self.issues.extend(formatted.issues);
			if let Some(content) = formatted.fixed {
				fixed.insert(path.clone(), content);
			}
			docs.insert(path.clone(), formatted.doc);
		}
		Ok((docs, fixed))
	}

	/// Reads and parses the document at `path` and, unless the formatter exclusions skip it, formats it. It only reads
	/// the pass, so documents can be formatted at the same time.
	fn format(&self, files: &dyn Files, path: &str, skip: bool) -> Result<Formatted, Error> {
		let read_error = |source| Error::Read {
			path: path.to_owned(),
			source,
		};
		let raw = files.read(path).map_err(read_error)?;
		let text =
			String::from_utf8(raw).map_err(|e| read_error(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
		let doc = Document::parse(&text);
		if skip {
			return Ok(Formatted {
				doc,
				fixed: None,
				issues: Vec::new(),
			});
		}
		let (formatted, issues) = self.apply_formatters(path, doc);
		match formatted {
			Changed::No(doc) => Ok(Formatted {
				doc,
				fixed: None,
				issues,
			}),
			Changed::Yes(doc) => Ok(Formatted {
				fixed: Some(doc.to_string()),
				doc,
				issues,
			}),
		}
	}

	/// Runs every enabled formatter over the document, each on the result of the one before, and returns the document
	/// they leave and, in [`Mode::Lint`], the lines each would change in the document as it is.
	fn apply_formatters(&self, path: &str, original: Document) -> (Changed, Vec<Issue>) {
		let mut current: Option<Document> = None;
		let mut issues = Vec::new();
		for f in formatters::ALL
			.iter()
			.filter(|f| self.config.formatter_enabled(f.about().name))
		{
			let about = f.about();
			let settings = &self.config.formatters.settings;
			let fix = f.apply(current.as_ref().unwrap_or(&original), settings);
			if self.mode == Mode::Lint {
				// A run reports what each formatter would change in the file as it is, so the lines are the file's
				// own; while no formatter has changed it, that is the fix just made.
				let reported = match &current {
					Some(_) => f.apply(&original, settings).changed,
					None => fix.changed.clone(),
				};
				issues.extend(
					reported
						.into_iter()
						.map(|line| Finding::new(path, line, about.message).found_by(about.name)),
				);
			}
			if !fix.changed.is_empty() {
				current = Some(Document::parse(&fix.lines.join("\n")));
			}
		}
		(current.map_or(Changed::No(original), Changed::Yes), issues)
	}

	/// Runs the enabled linters. The document linters run over the documents at the same time, spread over the
	/// cores; the repository linters, which need every document, run after them. The issues are recorded in the order
	/// of the linters, then of the paths, as a run one at a time would record them.
	fn lint(&mut self, repo: &Repo<'_>, paths: &[String], stop: &AtomicBool) -> Result<(), Error> {
		let excluded: HashMap<&str, bool> = paths
			.iter()
			.map(|p| (p.as_str(), self.record(self.config.linter_path_excluded(p))))
			.collect();
		let is_excluded = |p: &str| excluded.get(p).copied().unwrap_or(false);
		let config = self.config;
		let context = Context {
			repo,
			settings: &config.linters.settings,
			excluded: &is_excluded,
		};
		let enabled: Vec<Linter> = linters::ALL
			.iter()
			.copied()
			.filter(|l| config.linter_enabled(l.about().name))
			.collect();
		let found = parallel::map(paths, |path| {
			if stop.load(Ordering::Relaxed) {
				return vec![Err(Error::Stopped)];
			}
			lint_document(&enabled, &context, path)
		});
		if stop.load(Ordering::Relaxed) {
			return Err(Error::Stopped);
		}
		let mut found: Vec<_> = found.into_iter().map(Vec::into_iter).collect();
		for &linter in &enabled {
			let name = linter.about().name;
			for results in &mut found {
				if let Some(result) = results.next() {
					self.add(name, result?);
				}
			}
			if let Linter::Repository(linter) = linter {
				self.add(name, linter.check(&context));
			}
		}
		Ok(())
	}

	fn add(&mut self, source: &'static str, found: Vec<Finding>) {
		for finding in found {
			if !self.record(self.config.linter_excluded(&finding.path, source, &finding.message)) {
				self.issues.push(finding.found_by(source));
			}
		}
	}

	/// Notes the exclusions that matched, for warn-unused, and passes on whether they exclude.
	fn record(&mut self, (excluded, hits): (bool, Vec<ExclusionKey>)) -> bool {
		self.used.extend(hits);
		excluded
	}
}

/// Runs the document linters of `enabled` over the document at `path`, in order, and gives what each found, with a
/// placeholder for each repository linter, up to the first that fails.
fn lint_document(enabled: &[Linter], context: &Context<'_>, path: &str) -> Vec<Result<Vec<Finding>, Error>> {
	let doc = &context.repo.docs[path];
	let mut found = Vec::with_capacity(enabled.len());
	for linter in enabled {
		let result = match linter {
			Linter::Document(linter) => linter.check(context, path, doc),
			Linter::Repository(_) => Ok(Vec::new()),
		};
		let failed = result.is_err();
		found.push(result);
		if failed {
			break;
		}
	}
	found
}

/// A document as the formatters leave it: unchanged, or changed.
enum Changed {
	No(Document),
	Yes(Document),
}

#[cfg(all(test, feature = "disk"))]
mod tests;

#[cfg(test)]
mod in_memory {
	use std::collections::BTreeMap;

	use super::{Mode, check_documents};
	use crate::config::Config;

	#[test]
	fn documents_in_memory() {
		let documents = BTreeMap::from([
			(
				"doc.md".to_owned(),
				"# Title\n\nGo -> [here](guide.md) and [there](img.png).\n".to_owned(),
			),
			("img.png".to_owned(), String::new()),
		]);
		let checked = check_documents(&documents, &Config::default(), Mode::Lint).unwrap();
		let issues: Vec<String> = checked.issues.iter().map(ToString::to_string).collect();
		assert_eq!(
			issues,
			[
				"doc.md:3: ASCII arrow or missing space around an arrow (arrows)",
				"doc.md:3: broken link: guide.md (links)"
			]
		);
		assert_eq!(
			checked.fixed["doc.md"],
			"# Title\n\nGo → [here](guide.md) and [there](img.png).\n"
		);
	}
}
