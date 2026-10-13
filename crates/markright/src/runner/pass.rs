//! One pass over the documents: format every one, lint the result, and filter the issues through the exclusions.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{Checked, Mode, parallel};
use crate::Error;
use crate::config::{Config, ExclusionKey, Matched};
use crate::formatters;
use crate::fs::Files;
use crate::issue::{Finding, Issue};
use crate::linters::{self, Context, Kind, Linter, Repo};
use crate::markdown::Document;

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
	let Formatting { documents, fixed } = pass.format_all(files, paths, stop)?;
	let repo = Repo::new(files, documents);
	if mode == Mode::Lint {
		pass.lint(&repo, paths, stop)?;
	}
	let mut issues = pass.issues;
	issues.sort_by(|a, b| (&a.path, a.line, &a.message).cmp(&(&b.path, b.line, &b.message)));
	let unused = config
		.watched()
		.into_iter()
		.filter(|watched| !pass.used.contains(&watched.key))
		.map(|watched| watched.warning)
		.collect();
	Ok(Checked {
		issues,
		fixed,
		unused_exclusions: unused,
	})
}

/// What formatting every document gives: each document parsed as the linters see it, and the new content of those
/// the formatters change, by path.
struct Formatting {
	documents: BTreeMap<String, Document>,
	fixed: BTreeMap<String, String>,
}

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
	document: Document,
	fixed: Option<String>,
	issues: Vec<Issue>,
}

impl Pass<'_> {
	/// Reads, parses and formats every document, spread over the cores, then records the results in the order of
	/// `paths`, so the issues and the first error are the same however the work was spread.
	fn format_all(&mut self, files: &dyn Files, paths: &[String], stop: &AtomicBool) -> Result<Formatting, Error> {
		let skip: Vec<bool> = paths
			.iter()
			.map(|path| self.record(self.config.formatter_excluded(path)))
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
		let mut formatting = Formatting {
			documents: BTreeMap::new(),
			fixed: BTreeMap::new(),
		};
		for (path, result) in paths.iter().zip(results) {
			let formatted = result?;
			self.issues.extend(formatted.issues);
			if let Some(content) = formatted.fixed {
				formatting.fixed.insert(path.clone(), content);
			}
			formatting.documents.insert(path.clone(), formatted.document);
		}
		Ok(formatting)
	}

	/// Reads and parses the document at `path` and, unless the formatter exclusions skip it, formats it. It only reads
	/// the pass, so documents can be formatted at the same time.
	fn format(&self, files: &dyn Files, path: &str, skip: bool) -> Result<Formatted, Error> {
		let read_error = |source| Error::Read {
			path: path.to_owned(),
			source,
		};
		let raw = files.read(path).map_err(read_error)?;
		let text = String::from_utf8(raw).map_err(|e| read_error(io::Error::new(io::ErrorKind::InvalidData, e)))?;
		let document = Document::parse(&text);
		if skip {
			return Ok(Formatted {
				document,
				fixed: None,
				issues: Vec::new(),
			});
		}
		let (edit, issues) = self.apply_formatters(path, document);
		Ok(match edit {
			Edit::Unchanged(document) => Formatted {
				document,
				fixed: None,
				issues,
			},
			Edit::Changed(document) => Formatted {
				fixed: Some(document.to_string()),
				document,
				issues,
			},
		})
	}

	/// Runs every enabled formatter over the document, each on the result of the one before, and returns the document
	/// they leave and, in [`Mode::Lint`], the lines each would change in the document as it is.
	fn apply_formatters(&self, path: &str, original: Document) -> (Edit, Vec<Issue>) {
		let mut current: Option<Document> = None;
		let mut issues = Vec::new();
		for formatter in formatters::ALL
			.iter()
			.filter(|formatter| self.config.formatter_enabled(formatter.about().name))
		{
			let about = formatter.about();
			let settings = &self.config.formatters.settings;
			let fix = formatter.apply(current.as_ref().unwrap_or(&original), settings);
			if self.mode == Mode::Lint {
				// A run reports what each formatter would change in the file as it is. Until one has changed the
				// document, the fix just made already says that.
				let reported = match &current {
					Some(_) => formatter.apply(&original, settings).changed,
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
		(current.map_or(Edit::Unchanged(original), Edit::Changed), issues)
	}

	/// Runs the enabled linters. The document linters run over the documents at the same time, spread over the
	/// cores; the repository linters, which need every document, run after them. The issues are recorded in the order
	/// of the linters, then of the paths, as a run one at a time would record them.
	fn lint(&mut self, repo: &Repo<'_>, paths: &[String], stop: &AtomicBool) -> Result<(), Error> {
		let excluded: HashMap<&str, bool> = paths
			.iter()
			.map(|path| (path.as_str(), self.record(self.config.linter_path_excluded(path))))
			.collect();
		let is_excluded = |path: &str| excluded.get(path).copied().unwrap_or(false);
		let config = self.config;
		let context = Context {
			repo,
			settings: &config.linters.settings,
			excluded: &is_excluded,
		};
		let enabled: Vec<Linter> = linters::ALL
			.iter()
			.copied()
			.filter(|linter| config.linter_enabled(linter.about().name))
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
			if let Kind::Repository(linter) = linter.0 {
				self.add(name, linter.check(&context));
			}
		}
		Ok(())
	}

	/// Records what the rule `source` found, less what the linter exclusions drop.
	fn add(&mut self, source: &'static str, found: Vec<Finding>) {
		for finding in found {
			if !self.record(self.config.linter_excluded(&finding.path, source, &finding.message)) {
				self.issues.push(finding.found_by(source));
			}
		}
	}

	/// Notes the exclusions that matched, for warn-unused, and passes on whether they exclude.
	fn record(&mut self, matched: Matched) -> bool {
		self.used.extend(matched.by);
		matched.excluded
	}
}

/// Runs the document linters of `enabled` over the document at `path`, in order, and gives what each found, with a
/// placeholder for each repository linter, up to the first that fails.
fn lint_document(enabled: &[Linter], context: &Context<'_>, path: &str) -> Vec<Result<Vec<Finding>, Error>> {
	let document = &context.repo.docs[path];
	let mut found = Vec::with_capacity(enabled.len());
	for linter in enabled {
		let result = match linter.0 {
			Kind::Document(linter) => linter.check(context, path, document),
			Kind::Repository(_) => Ok(Vec::new()),
		};
		let failed = result.is_err();
		found.push(result);
		if failed {
			break;
		}
	}
	found
}

/// A document as the formatters leave it.
enum Edit {
	/// No formatter changed it.
	Unchanged(Document),
	/// The formatters changed it into this.
	Changed(Document),
}
