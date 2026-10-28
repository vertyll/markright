# Development

## Checks

```bash
bash scripts/check.sh
```

`scripts/check.sh` runs, in order:

1. `cargo fmt --check`, with the style of `rustfmt.toml`;
2. `cargo clippy`, with every pedantic lint, on the workspace, on the library without its features, and on the
   playground compiled to WebAssembly when that target is installed;
3. the tests of every crate, the schema, the examples and the tables of the documentation included, and every rule
   run over the examples of the CommonMark and GFM specifications, in `crates/markright-core/tests/spec`;
4. `cargo doc`, which fails on a broken link or tag in the API documentation;
5. `markright check` on this repository's own documents;
6. the type check, Biome's formatting and lints and the tests of the site's scripts and styles, with the scripts'
   coverage in `site/assets/lcov.info`.

The version of Rust is pinned in `rust-toolchain.toml`, so the checks run with the same compiler and the same lints
here and in CI; `rustup install` fetches it. Raising it is a change of its own, together with `rust-version` in
`Cargo.toml`. The lints are set once, in `[workspace.lints]` of `Cargo.toml`, and every crate takes them; a warning
fails the checks. CI runs the same script on every push and pull request to `main`, and on every tag, with the
WebAssembly target installed.

Node runs the checks of the site's scripts and styles; the tools they need, TypeScript, esbuild and Biome, and the
Inter typeface the site is set in are pinned in `site/assets/package.json` and fetched by `npm ci`. `npm run format` in
`site/assets` formats them as Biome expects.

With a SonarQube server at hand, `scripts/sonar.sh` runs the same checks, measures the coverage of the Rust code
with cargo-llvm-cov, and sends both to the server for analysis; `sonar-project.properties` says what it analyses.

Every script in `scripts/` says at its top what it does, how to call it and what it needs. They are Bash, so the
same file runs everywhere: on Windows, run them in Git Bash, which comes with Git, or in WSL.

## Adding a formatter or linter

1. Write it in a file of its own in `crates/markright-core/src/formatters` or `crates/markright-core/src/linters`, as
   a type that implements `Formatter`, `DocumentLinter` or `RepositoryLinter`:
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

## The documentation site

```bash
bash scripts/site.sh serve
```

`scripts/site.sh` writes the site's pages with `cargo xtask site`, compiles the playground to WebAssembly, lists the
licenses of the crates it includes, bundles the scripts, then runs Zola: `serve` serves the site at
<http://127.0.0.1:1111/> and rebuilds it on every change of `site/`, `build` writes it into `site/public`. It needs the
`wasm32-unknown-unknown` target, wasm-bindgen-cli at the version of the `wasm-bindgen` crate in `Cargo.lock`, Zola
0.23.6, and Node. Then

```bash
cargo xtask site check site/public https://vertyll.github.io/markright/
```

fails on a link of the built site that leads to no page or no anchor. CI builds and checks the site on every push and
pull request, and publishes it to GitHub Pages on every tag.

Every page that comes from a document or a rule is generated: a change goes into the document, the rule or
`xtask/src/site`, never into `site/content`, which is rewritten on every build. A new document listed in
`CONTENTS.md` needs a place on the site, in `xtask/src/site/documents.rs`, or the build fails. A setting for readers
is an entry in `site/data/settings.toml`, which the settings form shows, and a rule in `site/assets/styles` that
follows its attribute on the root element.

## Releasing

Versions follow Semantic Versioning: a new formatter or linter, or a changed default, is a minor version; a fix is a
patch. A change to the configuration format raises `version` in `.markright.yaml` and `.markright.toml` and is a
major version. Before `1.0.0`, a minor version may also change a default, and its release notes say so.

A release is an annotated tag on `main`, `vX.Y.Z`, made once the version in `[workspace.package]` of `Cargo.toml` is
raised, so `markright version` prints it. The tag runs the checks again; when they pass, CI builds the command for
every target below and publishes the release, with notes generated from the merged changes, and the documentation
site. Each archive is named `markright-<target>.tar.gz`, or `.zip` on Windows, and holds the command, its `LICENSE`
and `THIRD-PARTY-LICENSES.txt`, which `cargo xtask licenses` writes with the license of every crate built into it,
the Rust standard library included; `SHA256SUMS` lists their checksums.

| Target                       | System                                                           |
|------------------------------|------------------------------------------------------------------|
| `x86_64-unknown-linux-gnu`   | Linux on x86-64, as most CI runners                              |
| `x86_64-unknown-linux-musl`  | Linux on x86-64, linked statically: any distribution, Alpine too |
| `aarch64-unknown-linux-gnu`  | Linux on ARM64                                                   |
| `aarch64-unknown-linux-musl` | Linux on ARM64, linked statically: any distribution, Alpine too  |
| `aarch64-apple-darwin`       | macOS on Apple silicon                                           |
| `x86_64-apple-darwin`        | macOS on Intel                                                   |
| `x86_64-pc-windows-msvc`     | Windows on x86-64                                                |

Running CI by hand, from the Actions tab, builds every archive without releasing it, which tries a new target before a
tag depends on it.

> [!WARNING]
>
> A published release is fetched by the repositories that pin it. Its tag never moves, since that would change what
> they get without a trace; a correction is a new version.
