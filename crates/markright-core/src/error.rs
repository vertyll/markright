//! What can stop a run.

use std::io;
use std::path::PathBuf;

/// Why a run could not finish.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The run was interrupted before it finished; nothing was written.
    #[error("stopped: interrupted")]
    Stopped,
    /// Walking a directory for documents failed.
    #[error("listing documents: walking {}: {source}", path.display())]
    Walk {
        /// The directory.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// Opening the repository failed.
    #[error("opening {}: {source}", path.display())]
    Open {
        /// The root of the repository.
        path: PathBuf,
        /// Why.
        source: io::Error,
    },
    /// Reading a document failed.
    #[error("reading {path}: {source}")]
    Read {
        /// The document.
        path: String,
        /// Why.
        source: io::Error,
    },
    /// Finding out whether a link target exists failed.
    #[error("{linter}: resolving {path}: {source}")]
    Resolve {
        /// The linter that asked.
        linter: &'static str,
        /// The target.
        path: String,
        /// Why.
        source: io::Error,
    },
    /// Writing a formatted document failed.
    #[error("writing {path}: {source}")]
    Write {
        /// The document.
        path: String,
        /// Why.
        source: io::Error,
    },
}
