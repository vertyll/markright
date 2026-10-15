//! The command line: the commands, their arguments, and the configuration a command runs with.

use std::fmt;
use std::path::{Component, Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use markright_core::config::{self, Config, ConfigError};

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
    Fmt(Selection),
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
    pub(crate) selection: Selection,
}

/// The documents a command reports on or rewrites, in the repository it works on.
#[derive(Debug, Args)]
pub(crate) struct Selection {
    /// Documents or directories to take, relative to the current directory [default: the current directory, or the
    /// whole repository when --root names it]
    #[arg(value_name = "PATH")]
    pub(crate) paths: Vec<PathBuf>,
    #[command(flatten)]
    pub(crate) target: Target,
}

impl Selection {
    /// The paths, relative to `root`, with forward slashes, as the run takes them, empty for the root itself. With
    /// none given, the current directory, or the whole root when `--root` names it.
    pub(crate) fn relative(&self, root: &Path) -> Result<Vec<String>, PathError> {
        let current = [PathBuf::from(".")];
        let paths: &[PathBuf] = match (self.paths.is_empty(), self.target.root.is_some()) {
            (true, true) => return Ok(vec![String::new()]),
            (true, false) => &current,
            (false, _) => &self.paths,
        };
        let absolute_root = root.canonicalize().map_err(|_| PathError::Missing(root.to_owned()))?;
        paths
            .iter()
            .map(|path| {
                let absolute = path.canonicalize().map_err(|_| PathError::Missing(path.clone()))?;
                let inside = absolute.strip_prefix(&absolute_root).map_err(|_| PathError::Outside {
                    path: path.clone(),
                    root: root.to_owned(),
                })?;
                Ok(slashed(inside))
            })
            .collect()
    }
}

/// `path`, relative, with its parts joined by forward slashes.
fn slashed(path: &Path) -> String {
    let parts: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    parts.join("/")
}

/// Why a path given to a command cannot be taken.
#[derive(Debug)]
pub(crate) enum PathError {
    /// Nothing is there.
    Missing(PathBuf),
    /// It lies outside the repository.
    Outside {
        /// The path, as given.
        path: PathBuf,
        /// The root of the repository, as given.
        root: PathBuf,
    },
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(path) => write!(f, "{}: no such file or directory", path.display()),
            Self::Outside { path, root } => write!(
                f,
                "{} is outside the repository at {}; give its root with --root",
                path.display(),
                root.display()
            ),
        }
    }
}

impl std::error::Error for PathError {}

/// The repository a command works on, and its configuration.
#[derive(Debug, Args)]
pub(crate) struct Target {
    /// The root of the repository [default: the nearest directory up from the current one holding a configuration
    /// file or .git]
    #[arg(long, value_name = "DIR")]
    pub(crate) root: Option<PathBuf>,
    /// The configuration file, YAML or TOML by its extension [default: .markright.yaml or .markright.toml in the root]
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

impl Target {
    /// The root of the repository: the one `--root` names, or the one [`find_root`] finds from the current directory.
    pub(crate) fn root(&self) -> PathBuf {
        match &self.root {
            Some(root) => root.clone(),
            None => std::env::current_dir().map_or_else(|_| PathBuf::from("."), |here| find_root(&here)),
        }
    }

    /// The configuration the command runs with, and the file it was read from, if any: the one `--config` names, or
    /// the one the root holds, or the defaults.
    pub(crate) fn configuration(&self, root: &Path) -> Result<Found, ConfigError> {
        let path = match &self.config {
            Some(path) => Some(path.clone()),
            None => Config::locate(root)?,
        };
        let config = path.as_deref().map_or_else(|| Ok(Config::default()), Config::load)?;
        Ok(Found { config, path })
    }
}

/// The root of the repository holding `here`: the nearest directory, from `here` up, that holds a configuration file
/// or `.git`, so a command run in a subdirectory still reads the whole repository and its configuration; `here`
/// itself when none does.
pub(crate) fn find_root(here: &Path) -> PathBuf {
    let marks = [config::YAML_FILE, config::TOML_FILE, ".git"];
    here.ancestors()
        .find(|dir| marks.iter().any(|mark| dir.join(mark).exists()))
        .unwrap_or(here)
        .to_path_buf()
}

/// The configuration a command runs with, and the file it was read from, if any.
pub(crate) struct Found {
    /// The configuration.
    pub(crate) config: Config,
    /// The file it was read from; none when the defaults apply.
    pub(crate) path: Option<PathBuf>,
}
