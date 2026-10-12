//! The markright command: `markright run` reports, `markright fmt` rewrites.

mod cli;

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;
use clap::error::ErrorKind;
use markright::config::{self, Config};
use markright::output::{self, Rule, Streams};
use markright::runner::{Mode, Runner};
use markright::{formatters, linters, markdown};

use crate::cli::{Cli, Command, ConfigCommand, Found};

/// How the command ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Exit {
	/// All is well.
	Clean,
	/// `run` found issues.
	Issues,
	/// The command line or the configuration is wrong, or the run failed.
	Failed,
}

impl Exit {
	/// The exit code: 0, 1 or 2.
	fn code(self) -> u8 {
		match self {
			Self::Clean => 0,
			Self::Issues => 1,
			Self::Failed => 2,
		}
	}
}

impl From<Exit> for ExitCode {
	fn from(exit: Exit) -> Self {
		Self::from(exit.code())
	}
}

/// Why a command failed, said once, on the standard error.
type Failure = Box<dyn std::error::Error>;

/// Set by Ctrl+C: the run stops before it starts more work, and writes nothing.
static STOP: AtomicBool = AtomicBool::new(false);

fn main() -> ExitCode {
	// Without the handler, Ctrl+C ends the process at once, which writes nothing either.
	let _ = ctrlc::set_handler(|| STOP.store(true, Ordering::Relaxed));
	run(
		std::env::args_os(),
		&mut io::stdout().lock(),
		&mut io::stderr().lock(),
		&STOP,
	)
	.into()
}

/// Runs markright with `args`, writing to `stdout` and `stderr`, and says how it ended.
fn run(
	args: impl IntoIterator<Item = OsString>,
	stdout: &mut dyn Write,
	stderr: &mut dyn Write,
	stop: &AtomicBool,
) -> Exit {
	let cli = match Cli::try_parse_from(args) {
		Ok(cli) => cli,
		Err(e) => return usage(&e, stdout, stderr),
	};
	match execute(&cli.command, stdout, stderr, stop) {
		Ok(exit) => exit,
		Err(failure) => {
			// The standard error is the last place to say what went wrong; there is nothing to do when it fails too.
			let _ = writeln!(stderr, "markright: {failure}");
			Exit::Failed
		}
	}
}

/// Shows what clap made of a command line it does not run: the help or the version, or a usage error.
fn usage(e: &clap::Error, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Exit {
	let shown = if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
		write!(stdout, "{}", e.render())
	} else {
		write!(stderr, "{}", e.render())
	};
	if shown.is_ok() && !e.use_stderr() {
		Exit::Clean
	} else {
		Exit::Failed
	}
}

fn execute(
	command: &Command,
	stdout: &mut dyn Write,
	stderr: &mut dyn Write,
	stop: &AtomicBool,
) -> Result<Exit, Failure> {
	let target = match command {
		Command::Version => {
			writeln!(stdout, "markright {}", env!("CARGO_PKG_VERSION"))?;
			return Ok(Exit::Clean);
		}
		Command::Run(target)
		| Command::Fmt(target)
		| Command::Linters(target)
		| Command::Formatters(target)
		| Command::Config(ConfigCommand::Verify(target)) => target,
	};
	let Found { config, path } = target.configuration()?;
	match command {
		Command::Run(_) => lint(&target.root, &config, stdout, stderr, stop),
		Command::Fmt(_) => format(&target.root, &config, stdout, stop),
		Command::Linters(_) => {
			let rules = linters::ALL
				.iter()
				.map(|l| l.about())
				.map(|a| (a.name, a.description, config.linter_enabled(a.name)));
			list(stdout, rules)
		}
		Command::Formatters(_) => {
			let rules = formatters::ALL
				.iter()
				.map(|f| f.about())
				.map(|a| (a.name, a.description, config.formatter_enabled(a.name)));
			list(stdout, rules)
		}
		Command::Config(_) | Command::Version => verified(stdout, path.as_deref()),
	}
}

/// Says which configuration `config verify` found valid.
fn verified(stdout: &mut dyn Write, path: Option<&Path>) -> Result<Exit, Failure> {
	match path {
		Some(path) => writeln!(stdout, "markright: {} is valid", path.display())?,
		None => writeln!(
			stdout,
			"markright: no {} or {}; the defaults apply",
			config::YAML_FILE,
			config::TOML_FILE
		)?,
	}
	Ok(Exit::Clean)
}

fn lint(
	root: &Path,
	config: &Config,
	stdout: &mut dyn Write,
	stderr: &mut dyn Write,
	stop: &AtomicBool,
) -> Result<Exit, Failure> {
	let report = Runner {
		root,
		config,
		mode: Mode::Lint,
	}
	.run(stop)?;
	output::write(
		&config.output.formats,
		&report.issues,
		&rules(),
		&mut Streams { stdout, stderr },
	)?;
	for warning in &report.unused_exclusions {
		writeln!(stderr, "markright: warning: {warning}")?;
	}
	Ok(if report.issues.is_empty() {
		Exit::Clean
	} else {
		Exit::Issues
	})
}

fn format(root: &Path, config: &Config, stdout: &mut dyn Write, stop: &AtomicBool) -> Result<Exit, Failure> {
	let report = Runner {
		root,
		config,
		mode: Mode::Format,
	}
	.run(stop)?;
	for path in &report.formatted {
		writeln!(stdout, "formatted {path}")?;
	}
	writeln!(stdout, "markright: {} file(s) formatted", report.formatted.len())?;
	Ok(Exit::Clean)
}

/// Every formatter and linter, as the SARIF report lists them.
fn rules() -> Vec<Rule> {
	let formatters = formatters::ALL
		.iter()
		.map(|f| f.about())
		.map(|a| (a.name, a.description));
	let linters = linters::ALL.iter().map(|l| l.about()).map(|a| (a.name, a.description));
	formatters
		.chain(linters)
		.map(|(name, description)| Rule {
			name,
			description: markdown::plain(description),
		})
		.collect()
}

/// Lists rules by name, whether the configuration enables each, and what each does.
fn list<'r>(stdout: &mut dyn Write, rules: impl Iterator<Item = (&'r str, &'r str, bool)>) -> Result<Exit, Failure> {
	for (name, description, enabled) in rules {
		let state = if enabled { "enabled" } else { "disabled" };
		writeln!(stdout, "{name:<20} {state:<9} {}", markdown::plain(description))?;
	}
	Ok(Exit::Clean)
}

#[cfg(test)]
mod tests;
