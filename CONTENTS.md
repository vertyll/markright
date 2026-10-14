# Contents

Every document in this repository, the module it belongs to, and what it covers. Terms are defined in
[GLOSSARY.md](GLOSSARY.md), and the specifications behind them are in [STANDARDS.md](STANDARDS.md).

## Start here

| Document                  | Module | Kind              | Covers                                              |
|---------------------------|--------|-------------------|-----------------------------------------------------|
| [markright](README.md)    | —      | repository README | What the tool does and how to run it.               |
| [Glossary](GLOSSARY.md)   | —      | reference         | Every term the docs use, and where it is explained. |
| [Standards](STANDARDS.md) | —      | reference         | The specifications the tool follows.                |

## Overview

| Document                               | Module | Kind     | Covers                                                                                |
|----------------------------------------|--------|----------|---------------------------------------------------------------------------------------|
| [Rules](docs/rules.md)                 | —      | overview | The commands, every formatter and linter, and what each does.                         |
| [Configuration](docs/configuration.md) | —      | overview | `.markright.yaml` or `.markright.toml`: choosing what runs, settings, exclusions, CI. |
| [Architecture](docs/architecture.md)   | —      | overview | One run from configuration to issues, and the crates and modules.                     |
| [Development](docs/development.md)     | —      | overview | The checks, adding a formatter or linter and releasing a version.                     |
| [Contributing](CONTRIBUTING.md)        | —      | overview | Reporting a bug, proposing a rule, and how commits and pull requests are written.     |

## Mechanisms

| Document                                                    | Module | Kind      | Covers                                                                            |
|-------------------------------------------------------------|--------|-----------|-----------------------------------------------------------------------------------|
| [Document discovery](docs/mechanisms/document-discovery.md) | —      | mechanism | Which Markdown files are checked and which directories are skipped.               |
| [Code blocks](docs/mechanisms/code-blocks.md)               | —      | mechanism | Which lines are code: fenced, indented and HTML blocks, in lists and blockquotes. |
| [Anchors](docs/mechanisms/anchors.md)                       | —      | mechanism | How headings and HTML ids become anchors and how fragments are matched.           |
| [Display width](docs/mechanisms/display-width.md)           | —      | mechanism | How wide a character is when tables are aligned.                                  |
| [Protected spans](docs/mechanisms/protected-spans.md)       | —      | mechanism | What the formatters never rewrite.                                                |
