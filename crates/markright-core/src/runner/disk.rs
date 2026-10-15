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
        let paths = discover::markdown(self.root, stop)?;
        let disk = open(self.root)?;
        let checked = check(&disk, &paths, self.config, self.mode, stop)?;
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

/// The repository at `root`, opened for reading and writing inside it only.
fn open(root: &Path) -> Result<Disk, Error> {
    Dir::open_ambient_dir(root, ambient_authority())
        .map(Disk)
        .map_err(|source| Error::Open {
            path: root.to_owned(),
            source,
        })
}
