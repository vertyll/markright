//! The command line: the commands, their arguments, and the configuration a command runs with.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use markright_core::config::{Config, ConfigError};

/// Formats and lints the Markdown of a repository.
#[derive(Debug, Parser)]
#[command(name = "markright", version, arg_required_else_help = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Report formatting and lint issues; exit 1 when there are any
    Check(Check),
    /// Apply the formatters in place
    Fmt(Target),
    /// List the linters and whether the configuration enables them
    Linters(Target),
    /// List the formatters and whether the configuration enables them
    Formatters(Target),
    /// Work with the configuration file
    #[command(subcommand, arg_required_else_help = true)]
    Config(ConfigCommand),
    /// Print the version of markright
    Version,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    /// Check the configuration file and exit
    Verify(Target),
}

/// What `check` does besides reporting.
#[derive(Debug, Args)]
pub(crate) struct Check {
    /// Apply the formatters in place first, then report what they cannot fix
    #[arg(long)]
    pub(crate) fix: bool,
    #[command(flatten)]
    pub(crate) target: Target,
}

/// The repository a command works on, and its configuration.
#[derive(Debug, Args)]
pub(crate) struct Target {
    /// The root of the repository
    #[arg(default_value = ".")]
    pub(crate) root: PathBuf,
    /// The configuration file, YAML or TOML by its extension [default: .markright.yaml or .markright.toml in ROOT]
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

impl Target {
    /// The configuration the command runs with, and the file it was read from, if any: the one `--config` names, or
    /// the one the root holds, or the defaults.
    pub(crate) fn configuration(&self) -> Result<Found, ConfigError> {
        let path = match &self.config {
            Some(path) => Some(path.clone()),
            None => Config::locate(&self.root)?,
        };
        let config = path.as_deref().map_or_else(|| Ok(Config::default()), Config::load)?;
        Ok(Found { config, path })
    }
}

/// The configuration a command runs with, and the file it was read from, if any.
pub(crate) struct Found {
    /// The configuration.
    pub(crate) config: Config,
    /// The file it was read from; none when the defaults apply.
    pub(crate) path: Option<PathBuf>,
}
