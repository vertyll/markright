# Configuration

`markright` reads its configuration from the root it checks: `.markright.yaml`, in YAML, or `.markright.toml`, in
TOML, whichever the repository prefers; a root that holds both is an error. `--config` names another file, read as
TOML when its name ends in `.toml` and as YAML otherwise. Every repository that uses the tool keeps one, even when it
sets nothing beyond the defaults, so what runs is visible where the documents are and a change of defaults in a new
version shows up as a decision in the file.

## The smallest file

It turns every formatter and linter on.

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

Both say the same: every key below has the same name and meaning in either language, and this guide shows YAML.

## Rules of the file

A file must declare `version: "1"`; any key it sets replaces only that value, and a list replaces the whole default
list. Without the file every formatter and linter runs with the settings below. `markright config verify` checks a file
without running anything; an unknown key, name or regular expression, a name both enabled and disabled, or an
incomplete exclusion rule stops any command with exit code `2`, and the message names the line of the file:

```text
markright: .markright.yaml: line 5: unknown field `width`, expected one of max
```

Every key:

```yaml
version: "1"

output:
  formats:
    text:
      path: stdout
    checkstyle:
      path: markright.xml
    sarif:
      path: markright.sarif

linters:
  default: all
  enable: []
  disable: [line-length]
  settings:
    line-length:
      max: 120
    index:
      file: CONTENTS.md
    readme-links:
      file: README.md
      links: [CONTENTS.md, GLOSSARY.md, STANDARDS.md]
    alerts:
      types: [NOTE, TIP, IMPORTANT, WARNING, CAUTION]
    link-text:
      phrases: [click here, here, link, more, read more, this, this link]
  exclusions:
    warn-unused: true
    paths:
      - ^generated/
    paths-except: []
    rules:
      - path: ^docs/legacy\.md$
        linters: [line-length]
      - path-except: ^docs/
        text: "broken link: ../"

formatters:
  default: all
  enable: []
  disable: [tables]
  settings:
    list-marker:
      marker: "*"
    ordered-list-numbers:
      numbering: increasing
    emphasis-marker:
      marker: "*"
    fence-marker:
      marker: "`"
    horizontal-rule:
      style: "---"
  exclusions:
    warn-unused: true
    paths:
      - ^CHANGELOG\.md$
    paths-except: []
```

The same in TOML:

```toml
version = "1"

[output.formats.text]
path = "stdout"

[output.formats.checkstyle]
path = "markright.xml"

[output.formats.sarif]
path = "markright.sarif"

[linters]
default = "all"
enable = []
disable = ["line-length"]

[linters.settings.line-length]
max = 120

[linters.settings.index]
file = "CONTENTS.md"

[linters.settings.readme-links]
file = "README.md"
links = ["CONTENTS.md", "GLOSSARY.md", "STANDARDS.md"]

[linters.settings.alerts]
types = ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"]

[linters.settings.link-text]
phrases = ["click here", "here", "link", "more", "read more", "this", "this link"]

[linters.exclusions]
warn-unused = true
paths = ['^generated/']
paths-except = []

[[linters.exclusions.rules]]
path = '^docs/legacy\.md$'
linters = ["line-length"]

[[linters.exclusions.rules]]
path-except = '^docs/'
text = "broken link: ../"

[formatters]
default = "all"
enable = []
disable = ["tables"]

[formatters.settings.list-marker]
marker = "*"

[formatters.settings.ordered-list-numbers]
numbering = "increasing"

[formatters.settings.emphasis-marker]
marker = "*"

[formatters.settings.fence-marker]
marker = "`"

[formatters.settings.horizontal-rule]
style = "---"

[formatters.exclusions]
warn-unused = true
paths = ['^CHANGELOG\.md$']
paths-except = []
```

## Choosing what runs

`linters` and `formatters` each choose their own set. `default` is `all` or `none`; `enable` adds names to it and
`disable` removes them. `markright linters` and `markright formatters` list the names and what the configuration
makes of each. The names are in [Rules](rules.md).

## Settings

<!-- markright:settings -->

| Key                                                  | Meaning                                                                                                            | Default                                                                |
|------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| `formatters.settings.list-marker.marker`             | the marker of every item of an unordered list                                                                      | `-`                                                                    |
| `formatters.settings.ordered-list-numbers.numbering` | how the items of an ordered list are numbered                                                                      | `increasing`                                                           |
| `formatters.settings.emphasis-marker.marker`         | the character of emphasis and strong emphasis                                                                      | `*`                                                                    |
| `formatters.settings.fence-marker.marker`            | the character of the fences of code blocks                                                                         | `` ` ``                                                                |
| `formatters.settings.horizontal-rule.style`          | how a horizontal rule is written                                                                                   | `---`                                                                  |
| `linters.settings.link-text.phrases`                 | the texts that say nothing about where a link leads, compared without case and without the punctuation around them | `click here`, `here`, `link`, `more`, `read more`, `this`, `this link` |
| `linters.settings.alerts.types`                      | allowed alert types                                                                                                | `NOTE`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`                       |
| `linters.settings.line-length.max`                   | the longest line allowed, in characters                                                                            | `120`                                                                  |
| `linters.settings.index.file`                        | the document that must link every other one                                                                        | `CONTENTS.md`                                                          |
| `linters.settings.readme-links.file`                 | the README                                                                                                         | `README.md`                                                            |
| `linters.settings.readme-links.links`                | documents the README must link, when they exist                                                                    | `CONTENTS.md`, `GLOSSARY.md`, `STANDARDS.md`                           |

<!-- /markright:settings -->

Paths in the settings are relative to the root.

## Exclusions

Each section skips paths on its own, so a file can be linted but never formatted, or the other way round.

- `paths`: a matching document is excluded.
- `paths-except`: when set, a document that matches none of these is excluded as well, so a section can be limited
  to a few directories.
- `formatters.exclusions`: the formatters leave an excluded document as it is, and `check` does not report it as
  unformatted.
- `linters.exclusions`: no linter reports an issue in an excluded document, and `index` does not ask for it.
- `linters.exclusions.rules`: an issue is dropped when it matches every field a rule sets: `path` or `path-except`
  against the document, `linters` against the linter's name, `text` against the message. A rule sets at least two of
  those, and never both `path` and `path-except`, so it cannot silence a linter everywhere by accident; that is what
  `disable` is for.
- `warn-unused`: `check` prints a warning to standard error for every path pattern that matches no document and every
  rule that drops no issue, so stale exclusions surface. It does not change the exit code.

Paths and texts are regular expressions, in the syntax of Rust's `regex` crate, matched anywhere in the path relative
to the root, with `/` as the separator; `^` and `$` anchor them. Build and dependency directories are skipped before
any of this: [Document discovery](mechanisms/document-discovery.md).

## Output

`output.formats` chooses what `markright check` writes, and where: `stdout`, `stderr`, or a file relative to the working
directory. Without any format, it writes text to standard output.

| Format       | Content                                                                                             |
|--------------|-----------------------------------------------------------------------------------------------------|
| `text`       | one issue per line, `path:line: message (linter)`, then the count                                   |
| `checkstyle` | Checkstyle XML, for tools that read it, with `markright.<linter>` as source                         |
| `sarif`      | SARIF 2.1.0, with every formatter and linter as a rule, for tools that read static analysis results |

`markright fmt` always prints the files it changed, as text.

## Schema

`jsonschema/markright.jsonschema.json` describes the file, so an editor completes and checks it. Point the editor at
the schema of the version in use, for example with the YAML language server's modeline:

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/vertyll/markright/v0.1.0/jsonschema/markright.jsonschema.json
version: "1"
```

or by mapping `.markright.yaml` to that address in the editor's settings. For `.markright.toml`, a first line of
`#:schema` followed by the same address does the same for editors that use Taplo, the TOML language server.

The schema is generated from the types markright reads the file into, so an editor accepts exactly what markright
does: [Development](development.md#adding-a-formatter-or-linter).

## In CI

CI fetches the command of a pinned release, checks it against the release's `SHA256SUMS`, and fails the build on exit
code `1`:

```bash
version=v0.1.0
target=x86_64-unknown-linux-gnu
base=https://github.com/vertyll/markright/releases/download/$version
curl -fsSLO "$base/markright-$target.tar.gz"
curl -fsSL "$base/SHA256SUMS" | grep "markright-$target.tar.gz" | sha256sum -c -
tar -xzf "markright-$target.tar.gz"
./markright check
```

In a GitHub Actions workflow, those lines are the `run` of one step, after `actions/checkout`. To move to a newer
release, raise `version`. Each release has an archive for every system it is built for, named after its target; the
list is in [Development](development.md#releasing).
