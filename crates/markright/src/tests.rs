use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::run;

/// What a command did: its exit code, its standard output and its standard error.
#[derive(Debug, PartialEq, Eq)]
struct Done {
    code: u8,
    out: String,
    err: String,
}

/// Runs markright with `args`, in `root` when one is given as the last argument.
fn call(args: &[&str]) -> Done {
    call_until(args, false)
}

fn call_until(args: &[&str], stopped: bool) -> Done {
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let args = std::iter::once("markright").chain(args.iter().copied()).map(Into::into);
    let code = run(args, &mut stdout, &mut stderr, &AtomicBool::new(stopped)).code();
    Done {
        code,
        out: String::from_utf8(stdout).unwrap(),
        err: String::from_utf8(stderr).unwrap(),
    }
}

fn done(code: u8, out: &str, err: &str) -> Done {
    Done {
        code,
        out: out.to_owned(),
        err: err.to_owned(),
    }
}

fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}

#[test]
fn check_reports_and_fmt_fixes() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "# A\n\na -> b\n").unwrap();
    let report = "a.md:3: ASCII arrow or missing space around an arrow (arrows)\nmarkright: 1 issue(s)\n";
    assert_eq!(call(&["check", "--root", path(root.path())]), done(1, report, ""));
    assert_eq!(
        call(&["fmt", "--root", path(root.path())]),
        done(0, "formatted a.md\nmarkright: 1 file(s) formatted\n", "")
    );
    assert_eq!(
        call(&["check", "--root", path(root.path())]),
        done(0, "markright: 0 issues\n", "")
    );
    assert_eq!(fs::read_to_string(root.path().join("a.md")).unwrap(), "# A\n\na → b\n");
}

#[test]
fn check_fix_fixes_and_reports_the_rest() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "# A\n\na -> b\n").unwrap();
    fs::write(root.path().join("b.md"), "# B\n\na -> [b](gone.md)\n").unwrap();
    let report = "b.md:3: broken link: gone.md (links)\nmarkright: 1 issue(s)\n";
    assert_eq!(
        call(&["check", "--fix", "--root", path(root.path())]),
        done(1, report, "formatted a.md\nformatted b.md\n"),
        "the fixed files go to the standard error, so the report on the standard output keeps its format"
    );
    assert_eq!(fs::read_to_string(root.path().join("a.md")).unwrap(), "# A\n\na → b\n");
    fs::remove_file(root.path().join("b.md")).unwrap();
    assert_eq!(
        call(&["check", "--fix", "--root", path(root.path())]),
        done(0, "markright: 0 issues\n", "")
    );
}

#[test]
fn lists_follow_the_configuration() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join(".markright.yaml");
    fs::write(&config, "version: \"1\"\nlinters:\n  disable: [line-length]\n").unwrap();
    let listed = call(&["linters", "--root", path(root.path())]);
    let lines: Vec<&str> = listed.out.lines().collect();
    assert_eq!((listed.code, lines.len()), (0, 28));
    assert_eq!(
        lines[1],
        "anchors              enabled   a #fragment matches a heading of the target document"
    );
    assert_eq!(
        lines.iter().find(|line| line.starts_with("line-length ")),
        Some(&"line-length          disabled  a line is at most the configured number of characters")
    );
    let listed = call(&["formatters", "--config", path(&config)]);
    assert_eq!(
        listed.out.lines().find(|line| line.starts_with("tables ")),
        Some("tables               enabled   pads every column of a table to one width")
    );
}

#[test]
fn usage_errors() {
    for args in [
        &[][..],
        &["lint"],
        &["check", "--nope"],
        &["config", "check"],
        &["config"],
    ] {
        let failed = call(args);
        assert!(
            failed.code == 2 && failed.out.is_empty() && failed.err.contains("Usage: markright"),
            "{args:?}: {failed:?}"
        );
    }
}

#[test]
fn a_wrong_configuration_stops_every_command() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".markright.yaml"), "version: \"9\"\n").unwrap();
    let error = "markright: .markright.yaml: version must be \"1\", got \"9\"\n";
    for command in ["check", "fmt", "linters", "formatters"] {
        assert_eq!(
            call(&[command, "--root", path(root.path())]),
            done(2, "", error),
            "{command}"
        );
    }
}

#[test]
fn config_verify() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join(".markright.yaml");
    fs::write(&config, "version: \"1\"\nlinters:\n  disable: [links]\n").unwrap();
    let valid = format!("markright: {} is valid\n", config.display());
    assert_eq!(
        call(&["config", "verify", "--root", path(root.path())]),
        done(0, &valid, "")
    );
    fs::write(&config, "version: \"1\"\nlinters:\n  disable: [link]\n").unwrap();
    let error = "markright: .markright.yaml: linters: unknown name \"link\" (see markright linters)\n";
    assert_eq!(call(&["config", "verify", "-c", path(&config)]), done(2, "", error));
}

#[test]
fn without_a_configuration_the_defaults_apply() {
    let root = tempfile::tempdir().unwrap();
    let out = "markright: no .markright.yaml or .markright.toml; the defaults apply\n";
    assert_eq!(
        call(&["config", "verify", "--root", path(root.path())]),
        done(0, out, "")
    );
}

#[test]
fn a_toml_configuration() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join(".markright.toml"),
        "version = \"1\"\n\n[formatters]\ndisable = [\"arrows\"]\n",
    )
    .unwrap();
    fs::write(root.path().join("a.md"), "# A\n\na -> b\n").unwrap();
    assert_eq!(
        call(&["check", "--root", path(root.path())]),
        done(0, "markright: 0 issues\n", "")
    );
    fs::write(root.path().join(".markright.yaml"), "version: \"1\"\n").unwrap();
    let error = format!(
        "markright: both .markright.yaml and .markright.toml exist in {}; keep one\n",
        root.path().display()
    );
    assert_eq!(call(&["check", "--root", path(root.path())]), done(2, "", &error));
}

#[test]
fn output_formats_and_unused_warnings() {
    let root = tempfile::tempdir().unwrap();
    let sarif = root.path().join("out.sarif");
    let config = format!(
        "version: \"1\"\noutput:\n  formats:\n    text:\n      path: stdout\n    sarif:\n      path: {}\n\
		 linters:\n  exclusions:\n    warn-unused: true\n    paths: ['^nothing/']\n",
        path(&sarif)
    );
    fs::write(root.path().join(".markright.yaml"), config).unwrap();
    fs::write(root.path().join("a.md"), "# A\n\n[x](gone.md)\n").unwrap();
    let out = "a.md:3: broken link: gone.md (links)\nmarkright: 1 issue(s)\n";
    let err = "markright: warning: linters.exclusions.paths[0]: \"^nothing/\" matches no document\n";
    assert_eq!(call(&["check", "--root", path(root.path())]), done(1, out, err));
    let log: serde_json::Value = serde_json::from_str(&fs::read_to_string(&sarif).unwrap()).unwrap();
    assert_eq!(log["runs"][0]["results"][0]["ruleId"], "links");
}

#[test]
fn help_and_version() {
    for arg in ["help", "-h", "--help"] {
        let shown = call(&[arg]);
        assert!(
            shown.code == 0 && shown.err.is_empty() && shown.out.starts_with("Formats and lints"),
            "{arg}: {shown:?}"
        );
    }
    let version = format!("markright {}\n", env!("CARGO_PKG_VERSION"));
    for arg in ["version", "--version", "-V"] {
        assert_eq!(call(&[arg]), done(0, &version, ""), "{arg}");
    }
}

#[test]
fn an_unknown_command_is_reported_before_the_configuration() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".markright.yaml"), "broken: [\n").unwrap();
    let failed = call(&["lint", "--root", path(root.path())]);
    assert!(
        failed.code == 2 && failed.err.starts_with("error: unrecognized subcommand 'lint'"),
        "{failed:?}"
    );
}

#[test]
fn an_interrupted_run_fails_and_writes_nothing() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "a -> b\n").unwrap();
    assert_eq!(
        call_until(&["fmt", "--root", path(root.path())], true),
        done(2, "", "markright: stopped: interrupted\n")
    );
    assert_eq!(fs::read_to_string(root.path().join("a.md")).unwrap(), "a -> b\n");
}

#[test]
fn a_closed_output_fails_the_command() {
    /// Refuses every write, as a closed pipe does.
    struct Closed;

    impl io::Write for Closed {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut stderr = Vec::new();
    let args = ["markright", "linters"].map(Into::into);
    assert_eq!(run(args, &mut Closed, &mut stderr, &AtomicBool::new(false)).code(), 2);
    assert_eq!(String::from_utf8(stderr).unwrap(), "markright: broken pipe\n");
}

#[test]
fn paths_choose_the_documents_reported_and_formatted() {
    let root = tempfile::tempdir().unwrap();
    let docs = root.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(root.path().join("a.md"), "# A\n\na -> b\n").unwrap();
    fs::write(docs.join("b.md"), "# B\n\nb -> c, [A](../a.md#a)\n").unwrap();
    fs::write(docs.join("c.md"), "# C\n\n[B](b.md#gone)\n").unwrap();
    let base = ["--root", path(root.path())];
    let a = root.path().join("a.md");

    let report = "a.md:3: ASCII arrow or missing space around an arrow (arrows)\nmarkright: 1 issue(s)\n";
    assert_eq!(
        call(&["check", path(&a), base[0], base[1]]),
        done(1, report, ""),
        "one file"
    );
    let report = "docs/b.md:3: ASCII arrow or missing space around an arrow (arrows)\n\
                  docs/c.md:3: broken anchor: b.md#gone (anchors)\nmarkright: 2 issue(s)\n";
    assert_eq!(
        call(&["check", path(&docs), base[0], base[1]]),
        done(1, report, ""),
        "a directory, whose links into documents outside it still resolve"
    );

    assert_eq!(
        call(&["fmt", path(&docs.join("b.md")), base[0], base[1]]),
        done(0, "formatted docs/b.md\nmarkright: 1 file(s) formatted\n", "")
    );
    assert_eq!(
        fs::read_to_string(&a).unwrap(),
        "# A\n\na -> b\n",
        "a document not chosen is not written"
    );
    assert_eq!(
        fs::read_to_string(docs.join("b.md")).unwrap(),
        "# B\n\nb → c, [A](../a.md#a)\n"
    );
}

#[test]
fn a_path_that_cannot_be_taken_stops_the_command() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "# A\n").unwrap();
    fs::write(root.path().join("notes.txt"), "a\n").unwrap();
    let base = ["--root", path(root.path())];

    let missing = root.path().join("gone.md");
    let error = format!("markright: {}: no such file or directory\n", missing.display());
    assert_eq!(call(&["check", path(&missing), base[0], base[1]]), done(2, "", &error));

    let error = format!(
        "markright: {} is outside the repository at {}; give its root with --root\n",
        elsewhere.path().display(),
        root.path().display()
    );
    assert_eq!(
        call(&["fmt", path(elsewhere.path()), base[0], base[1]]),
        done(2, "", &error)
    );

    let error = "markright: notes.txt is not a document: markright reads the Markdown files Git tracks or would add, \
                 outside build and dependency directories and documents.exclude\n";
    let notes = root.path().join("notes.txt");
    assert_eq!(call(&["check", path(&notes), base[0], base[1]]), done(2, "", error));
}

#[test]
fn unused_exclusions_are_warned_of_only_for_the_whole_repository() {
    let root = tempfile::tempdir().unwrap();
    let config = "version: \"1\"\nlinters:\n  exclusions:\n    paths: [\"^gone/\"]\n    warn-unused: true\n";
    fs::write(root.path().join(".markright.yaml"), config).unwrap();
    fs::write(root.path().join("a.md"), "# A\n").unwrap();
    fs::write(root.path().join("b.md"), "# B\n").unwrap();
    let a = root.path().join("a.md");
    let whole = call(&["check", "--root", path(root.path())]);
    assert!(whole.err.contains("warning"), "{whole:?}");
    assert_eq!(
        call(&["check", path(&a), "--root", path(root.path())]),
        done(0, "markright: 0 issues\n", "")
    );
}

#[test]
fn the_root_is_the_nearest_directory_with_a_configuration_or_git() {
    let top = tempfile::tempdir().unwrap();
    let repo = top.path().join("repo");
    let deep = repo.join("docs/guides");
    fs::create_dir_all(&deep).unwrap();
    assert_eq!(crate::cli::find_root(&deep), deep, "nothing marks a root");

    fs::create_dir(repo.join(".git")).unwrap();
    assert_eq!(crate::cli::find_root(&deep), repo);

    fs::write(repo.join("docs/.markright.toml"), "version = \"1\"\n").unwrap();
    assert_eq!(
        crate::cli::find_root(&deep),
        repo.join("docs"),
        "a configuration nearer than .git makes its own root"
    );
}

#[test]
fn excluded_documents_are_neither_checked_nor_linked_into() {
    let root = tempfile::tempdir().unwrap();
    let vendor = root.path().join("vendor");
    fs::create_dir(&vendor).unwrap();
    fs::write(
        root.path().join(".markright.toml"),
        "version = \"1\"\n\n[documents]\nexclude = ['^vendor/']\n",
    )
    .unwrap();
    fs::write(
        root.path().join("a.md"),
        "# A\n\nSee [the notes](vendor/notes.md#anything).\n",
    )
    .unwrap();
    fs::write(vendor.join("notes.md"), "no heading -> here\n").unwrap();
    let base = ["--root", path(root.path())];

    assert_eq!(call(&["check", base[0], base[1]]), done(0, "markright: 0 issues\n", ""));
    assert_eq!(
        call(&["fmt", base[0], base[1]]),
        done(0, "markright: 0 file(s) formatted\n", "")
    );

    let notes = vendor.join("notes.md");
    let error = "markright: vendor/notes.md is not a document: markright reads the Markdown files Git tracks or would \
                 add, outside build and dependency directories and documents.exclude\n";
    assert_eq!(call(&["check", path(&notes), base[0], base[1]]), done(2, "", error));
}

#[test]
fn links_and_anchors_are_checked_only_against_their_targets() {
    let root = tempfile::tempdir().unwrap();
    let docs = root.path().join("docs");
    fs::create_dir(&docs).unwrap();
    fs::write(root.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
    fs::write(docs.join("guide.md"), "# Guide\n").unwrap();
    fs::write(
        root.path().join("README.md"),
        "# R\n\n[a](CHANGELOG.md#gone), [b](docs/guide.md#gone), [c](generated/api.md), [d](docs/gone.md)\n",
    )
    .unwrap();
    let base = ["--root", path(root.path())];
    let everything = "README.md:3: broken anchor: CHANGELOG.md#gone (anchors)\n\
                      README.md:3: broken anchor: docs/guide.md#gone (anchors)\n\
                      README.md:3: broken link: docs/gone.md (links)\n\
                      README.md:3: broken link: generated/api.md (links)\n\
                      markright: 4 issue(s)\n";
    assert_eq!(call(&["check", base[0], base[1]]), done(1, everything, ""));

    let config = "version = \"1\"\n\n[linters.settings.anchors]\ntargets = ['^docs/']\n\n\
                  [linters.settings.links]\ntargets = ['^docs/']\n";
    fs::write(root.path().join(".markright.toml"), config).unwrap();
    let narrowed = "README.md:3: broken anchor: docs/guide.md#gone (anchors)\n\
                    README.md:3: broken link: docs/gone.md (links)\n\
                    markright: 2 issue(s)\n";
    assert_eq!(call(&["check", base[0], base[1]]), done(1, narrowed, ""));
}
