//! A run over a repository on disk, which writes what the formatters change.

use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

use super::{Mode, check};
use crate::Error;
use crate::config::Config;
use crate::discover;
use crate::fs::Files;
use crate::issue::Issue;

/// Makes one pass over the repository at `root` with `config` each time [`Runner::run`] is called.
#[derive(Debug)]
pub struct Runner<'a> {
    /// The root of the repository.
    pub root: &'a Path,
    /// The configuration.
    pub config: &'a Config,
    /// What to do with the formatters' changes.
    pub mode: Mode,
}

/// What a run found: the issues, sorted by path and line, the documents it rewrote, and the warnings for watched
/// exclusions that matched nothing.
#[derive(Debug, Default)]
pub struct Report {
    /// The issues: in [`Mode::Lint`] every one, in [`Mode::Fix`] those no formatter fixes, in [`Mode::Format`] none.
    pub issues: Vec<Issue>,
    /// The documents rewritten, sorted; none in [`Mode::Lint`].
    pub formatted: Vec<String>,
    /// A warning for each watched exclusion that matched nothing.
    pub unused_exclusions: Vec<String>,
}

/// The repository on disk, opened so no path can lead out of it, symbolic links included.
struct Disk(Dir);

impl Files for Disk {
    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        self.0.read(path)
    }

    fn exists(&self, path: &str) -> io::Result<bool> {
        self.0.try_exists(path)
    }
}

impl Runner<'_> {
    /// Lists the repository's documents and checks them. In [`Mode::Lint`] it changes nothing; in [`Mode::Format`]
    /// and [`Mode::Fix`] it writes every changed document back, once every document is checked. It stops before
    /// writing anything once `stop` is set.
    ///
    /// # Errors
    ///
    /// Returns why the documents could not be listed, read or written, or [`Error::Stopped`].
    pub fn run(&self, stop: &AtomicBool) -> Result<Report, Error> {
        self.run_on(&[String::new()], stop)
    }

    /// Does what [`run`](Self::run) does, but reports on and writes only the documents at `paths` and under them:
    /// each a document or a directory, relative to the root, with forward slashes, and empty for the root itself.
    /// Every document is still read, so links and anchors into the others resolve; the warnings for exclusions that
    /// matched nothing come only when the paths take in every document.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotADocument`] for a file in `paths` that is not one of the repository's documents, why the
    /// documents could not be listed, read or written, or [`Error::Stopped`].
    pub fn run_on(&self, paths: &[String], stop: &AtomicBool) -> Result<Report, Error> {
        let documents: Vec<String> = discover::markdown(self.root, stop)?
            .into_iter()
            .filter(|path| self.config.is_document(path))
            .collect();
        let disk = open(self.root)?;
        let selected = select(&disk, &documents, paths)?;
        let checked = check(&disk, &documents, &selected, self.config, self.mode, stop)?;
        if self.mode == Mode::Lint {
            return Ok(Report {
                issues: checked.issues,
                formatted: Vec::new(),
                unused_exclusions: checked.unused_exclusions,
            });
        }
        for (path, content) in &checked.fixed {
            disk.0.write(path, content).map_err(|source| Error::Write {
                path: path.clone(),
                source,
            })?;
        }
        Ok(Report {
            issues: checked.issues,
            formatted: checked.fixed.into_keys().collect(),
            unused_exclusions: checked.unused_exclusions,
        })
    }
}

/// The `documents` at or under `paths`, in the order of `documents`. A path naming a file that is no document is an
/// error, so a typo or an ignored file is not passed over in silence; a directory without documents selects none.
fn select(disk: &Disk, documents: &[String], paths: &[String]) -> Result<Vec<String>, Error> {
    let within = |document: &str, path: &str| {
        path.is_empty() || document == path || document.strip_prefix(path).is_some_and(|rest| rest.starts_with('/'))
    };
    for path in paths {
        let found = documents.iter().any(|document| within(document, path));
        if !found && !path.is_empty() && disk.0.is_file(path) {
            return Err(Error::NotADocument { path: path.clone() });
        }
    }
    Ok(documents
        .iter()
        .filter(|document| paths.iter().any(|path| within(document, path)))
        .cloned()
        .collect())
}

/// The repository at `root`, opened for reading and writing inside it only.
fn open(root: &Path) -> Result<Disk, Error> {
    Dir::open_ambient_dir(root, ambient_authority())
        .map(Disk)
        .map_err(|source| Error::Open {
            path: root.to_owned(),
            source,
        })
}
