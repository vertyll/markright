//! Formats and lints the Markdown of a repository.

pub mod config;
#[cfg(feature = "disk")]
pub mod discover;
mod error;
pub mod formatters;
pub mod fs;
pub mod issue;
pub mod linters;
pub mod markdown;
pub mod output;
pub mod runner;

pub use error::Error;
