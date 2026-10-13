<p align="center">
    <img alt="" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white">
    <img alt="" src="https://img.shields.io/badge/Markdown-000000?style=for-the-badge&logo=markdown&logoColor=white">
</p>

# markright

`markright` keeps a repository's Markdown consistent.

- **Formatters fix** ASCII arrows, trailing whitespace, list markers, unaligned tables, repeated blank lines and the
  end of the file.
- **Linters report** broken links and anchors, unused reference definitions, malformed alerts and blockquotes, code
  blocks left open or without a language, skipped heading levels, images without alternative text, table rows with
  extra cells, long lines and documents missing from the index.

Every rule, with an example, is in [Rules](docs/rules.md). The documentation is also a site, with a playground that
runs markright on a document in the browser: <https://vertyll.github.io/markright/>.

## Install

Every release ships the `markright` command built for Linux, macOS and Windows. In CI, fetch the one for the runner and
run it:

```bash
curl -fsSL https://github.com/vertyll/markright/releases/download/v0.1.0/markright-x86_64-unknown-linux-gnu.tar.gz | tar -xz
./markright check
```

or build a pinned version with Cargo:

```bash
cargo install --locked --git https://github.com/vertyll/markright --tag v0.1.0 markright-cli
```

## Quick start

```bash
markright fmt
markright check
```

`fmt` applies the formatters in place. `check` changes nothing and reports every issue, the lines `fmt` would change
included, one per line, and exits with `1` when there is any:

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
keep one. The rest is in [Configuration](docs/configuration.md), and
every formatter and linter in [Rules](docs/rules.md).

## Documentation

- [Contents](CONTENTS.md) – every document in the repository, the module it belongs to, and what it covers.
- [Glossary](GLOSSARY.md) – every term the docs use, and where it is explained.
- [Standards](STANDARDS.md) – the specifications the tool follows.

## License

markright is released under the [MIT License](LICENSE).
