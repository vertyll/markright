# Standards

The specifications this tool follows, what for, and where the documentation covers it.

| Standard                        | Title                                      | Used for                                                                        | Explained in                                                  |
|---------------------------------|--------------------------------------------|---------------------------------------------------------------------------------|---------------------------------------------------------------|
| [CommonMark 0.31.2][commonmark] | CommonMark Spec                            | parsing every document: blocks, code, links and their positions                 | [Code blocks](docs/mechanisms/code-blocks.md)                 |
| [GFM 0.29][gfm]                 | GitHub Flavored Markdown Spec              | tables, strikethrough, task lists, and which plain URLs `bare-urls` makes links | [Rules: Formatters](docs/rules.md#formatters)                 |
| [GitHub alerts][alerts]         | GitHub alert syntax                        | the allowed alert types                                                         | [Rules: Linters](docs/rules.md#linters)                       |
| [GitHub footnotes][footnotes]   | GitHub footnote syntax                     | the footnotes the parser reads, which `footnotes` checks                        | [Rules: Linters](docs/rules.md#linters)                       |
| [UAX #11][uax11]                | Unicode East Asian Width                   | which characters are two columns wide                                           | [Display width](docs/mechanisms/display-width.md)             |
| [UTS #51][uts51]                | Unicode Emoji                              | emoji presentation and U+FE0F                                                   | [Display width](docs/mechanisms/display-width.md)             |
| [YAML 1.2][yaml]                | YAML Ain't Markup Language 1.2             | `.markright.yaml` and YAML front matter                                         | [Configuration](docs/configuration.md)                        |
| [TOML 1.1.0][toml]              | Tom's Obvious Minimal Language 1.1.0       | `.markright.toml` and TOML front matter                                         | [Configuration](docs/configuration.md)                        |
| [SARIF 2.1.0][sarif]            | Static Analysis Results Interchange Format | the `sarif` output                                                              | [Configuration: Output](docs/configuration.md#output)         |
| [Checkstyle XML][checkstyle]    | Checkstyle report format                   | the `checkstyle` output                                                         | [Configuration: Output](docs/configuration.md#output)         |
| [JSON Schema 2020-12][schema]   | JSON Schema                                | describing the configuration file for editors                                   | [Configuration: Schema](docs/configuration.md#schema)         |
| [Rust regex syntax][regex]      | Syntax of the regex crate                  | exclusion paths and texts                                                       | [Configuration: Exclusions](docs/configuration.md#exclusions) |
| [SemVer 2.0.0][semver]          | Semantic Versioning                        | release tags                                                                    | [Development: Releasing](docs/development.md#releasing)       |
| [Conventional Commits][commits] | Conventional Commits 1.0.0                 | commit messages and pull request titles                                         | [Contributing](CONTRIBUTING.md)                               |

[commonmark]: https://spec.commonmark.org/0.31.2/
[gfm]: https://github.github.com/gfm/
[alerts]: https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts
[footnotes]: https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#footnotes
[uax11]: https://www.unicode.org/reports/tr11/
[uts51]: https://www.unicode.org/reports/tr51/
[yaml]: https://yaml.org/spec/1.2.2/
[toml]: https://toml.io/en/v1.1.0
[sarif]: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html
[checkstyle]: https://checkstyle.org/
[schema]: https://json-schema.org/draft/2020-12
[regex]: https://docs.rs/regex/latest/regex/#syntax
[semver]: https://semver.org/spec/v2.0.0.html
[commits]: https://www.conventionalcommits.org/en/v1.0.0/
