# Development

## Checks

```bash
bash scripts/check.sh
```

`scripts/check.sh` runs, in order:

1. `cargo fmt --check`, with the style of `rustfmt.toml`;
2. `cargo clippy`, with every pedantic lint, on the workspace, on the library without its features, and on the
   playground compiled to WebAssembly when that target is installed;
3. the tests of every crate, the schema, the examples and the tables of the documentation included;
4. `markright run` on this repository's own documents.

The lints are set once, in `[workspace.lints]` of `Cargo.toml`, and every crate takes them; a warning fails the
checks. CI runs the same script on every push and pull request to `main`, and on every tag, with the WebAssembly
target installed.

Every script in `scripts/` says at its top what it does, how to call it and what it needs. They are Bash, so the
same file runs everywhere: on Windows, run them in Git Bash, which comes with Git, or in WSL.

## Adding a formatter or linter

1. Write it in a file of its own in `crates/markright/src/formatters` or `crates/markright/src/linters`, as a type
   that implements `Formatter`, `DocumentLinter` or `RepositoryLinter`:
   [Architecture: Registries](architecture.md#registries).
2. Add it to `ALL` in `formatters.rs` or `linters.rs`, next to the directory; its place in the list decides when it
   runs.
3. When it takes settings, give them a type in `settings.rs` of the same directory, as the others there: it derives
   `Serialize`, `Deserialize` and `JsonSchema` with `#[serde(default, deny_unknown_fields)]`, implements `Default`
   with its defaults, and documents each key in a comment, which becomes its description in the schema. Let the
   types refuse what makes no sense, as `NonZeroUsize` refuses a limit of zero. Add a field for it to that file's
   `Settings`, under the rule's name; the rule reads it from the settings it gets, as `settings.line_length.max`.
4. Run `cargo xtask schema`, which writes `jsonschema/markright.jsonschema.json` from the configuration's types, the
   rule names included; a test fails while the file is out of date. Show each new setting in the examples under
   "Every key" in [Configuration](configuration.md), in YAML and in TOML: a test fails while the YAML example leaves
   out a key of the schema, and another while the two say different things.
5. Give its `about()` an `example`: for a formatter, a document it changes; for a linter, a document it reports and
   the same document fixed. A test runs every example. Set `since` to the release that will ship it.
6. Test what it reports or rewrites and what it leaves alone, next to the others.
7. Write its `description` as one line of Markdown, with code spans as the only markup, and run
   `cargo xtask tables`: it writes the formatter and linter tables of [Rules](rules.md) from the registry, and the
   settings table of [Configuration](configuration.md#settings) from the schema. A test fails while they are out of
   date.

Nothing else changes: the runner, the commands, the configuration's name checks and the tables of Rules and
Configuration follow the registry and the schema.

## Releasing

A release is an annotated tag on `main` that follows Semantic Versioning: a new formatter or linter, or a changed
default, is a minor version; a fix is a patch. A change to the configuration format raises `version` in
`.markright.yaml` and `.markright.toml` and is a major version. The version in `[workspace.package]` of
`Cargo.toml` is raised first, so `markright version` prints it.

```bash
git push origin main && git tag -a vX.Y.Z -m "vX.Y.Z" && git push origin vX.Y.Z
```

The tag runs the checks again; when they pass, CI builds the command for every target below and publishes the
release with notes generated from the merged changes. Each archive is named `markright-<target>.tar.gz`, or `.zip` on
Windows, and `SHA256SUMS` lists their checksums.

| Target                      | System                              |
|-----------------------------|-------------------------------------|
| `x86_64-unknown-linux-gnu`  | Linux on x86-64, as most CI runners |
| `aarch64-unknown-linux-gnu` | Linux on ARM64                      |
| `aarch64-apple-darwin`      | macOS on Apple silicon              |
| `x86_64-pc-windows-msvc`    | Windows on x86-64                   |

> [!WARNING]
>
> A published release is fetched by the repositories that pin it. Moving its tag changes what they get without a
> trace; a correction is a new version.

After a release, the repositories that use the tool raise the version in their CI.
