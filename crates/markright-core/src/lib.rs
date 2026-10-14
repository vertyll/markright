//! Formats and lints the Markdown of a repository: the library behind the `markright` command and its playground.

pub mod config;
#[cfg(feature = "disk")]
mod discover;
mod error;
pub mod formatters;
mod fs;
pub mod issue;
pub mod linters;
pub mod markdown;
pub mod output;
pub mod runner;

pub use error::Error;

/// The release every rule has been part of since: the first.
pub(crate) const FIRST_RELEASE: &str = "v0.1.0";
