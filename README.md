<p align="center">
    <img alt="" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white">
</p>

# markright

`markright` keeps a repository's Markdown consistent.

- **Formatters fix** tabs, trailing whitespace, how headings, lists, emphasis, code fences, horizontal rules and
  blockquotes are written, the numbers of ordered lists, the blank lines around blocks and in a row, URLs and spaces
  in links and code, ASCII arrows, tables without their outer pipes or unaligned, and the end of the file.
- **Linters report** broken links and anchors, unused, repeated or reversed references, links that say nothing,
  malformed alerts and blockquotes, code blocks left open, indented or without a language, missing, repeated or skipped
  headings or headings that end like a sentence, emphasis in place of a heading, unevenly indented lists, images without
  alternative text, table rows with extra cells, long lines and documents missing from the index.

Every rule, with an example, is in [Rules](docs/rules.md). The documentation is also a site, with a playground that
runs markright on a document in the browser: <https://vertyll.github.io/markright/>.

## Install

Every release ships the `markright` command built for Linux, macOS and Windows. In CI, fetch the one for the runner and
run it:

```bash
curl -fsSL https://github.com/vertyll/markright/releases/download/v0.2.2/markright-x86_64-unknown-linux-gnu.tar.gz | tar -xz
./markright check
```

or build it with Cargo, from crates.io or from a tag of the repository:

```bash
cargo install --locked markright
cargo install --locked --git https://github.com/vertyll/markright --tag v0.2.2 markright
```

## Quick start

```bash
markright fmt
markright check --fix
markright check
markright check README.md docs
```

From the root of a repository, each command takes every document, as `markright check .` does; given files or
directories, it takes only those, while still resolving links and anchors against the whole repository. `fmt` applies
the formatters in place. `check --fix` does the same and reports what they cannot fix. `check` changes nothing and
reports every issue, the lines a formatter would change included, one per line, and exits with `1` when there is any;
it is the one CI runs:

```text
docs/guide.md:3: ASCII arrow or missing space around an arrow (arrows)
docs/guide.md:5: broken anchor: #instalation (anchors)
docs/guide.md:5: broken link: setup.md (links)
docs/guide.md:9: code block without a language (fence-language)
markright: 4 issue(s)
```

Every issue names the rule that found it.

## Configuration

A repository chooses its formatters and linters, their settings and the paths each one skips in one file at its
root, written in YAML or in TOML. The smallest one turns everything on.

### YAML: `.markright.yaml`

```yaml
version: "1"

linters:
  default: all

formatters:
  default: all
```

### TOML: `.markright.toml`

```toml
version = "1"

[linters]
default = "all"

[formatters]
default = "all"
```

Both files above set the same. A repository has one of them: with both at its root, markright stops and asks to
keep one. The rest is in [Configuration](docs/configuration.md), and every formatter and linter in
[Rules](docs/rules.md).

## Documentation

- [Contents](CONTENTS.md) – every document in the repository, the module it belongs to, and what it covers.
- [Glossary](GLOSSARY.md) – every term the docs use, and where it is explained.
- [Standards](STANDARDS.md) – the specifications the tool follows.
- [Contributing](CONTRIBUTING.md) – reporting a bug, proposing a rule, and sending a change.
- [Security](SECURITY.md) – reporting a vulnerability privately.

## License

markright is released under the [MIT License](LICENSE).
