# Glossary

Every term the documentation uses without defining it on the spot, and where it is explained. The specifications behind
them are in [STANDARDS.md](STANDARDS.md).

| Term              | Meaning                                                                                                                         | Explained in                                                                        |
|-------------------|---------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| Alert             | A blockquote that starts with a tag like `[!NOTE]`, which GitHub renders as a colored callout.                                  | [Rules: Linters](docs/rules.md#linters)                                             |
| Anchor            | The fragment GitHub derives from a heading, which a `#fragment` link must match.                                                | [Anchors](docs/mechanisms/anchors.md)                                               |
| Code block        | A fenced or indented code block, or an HTML block, as CommonMark parses it; no rule looks inside.                               | [Code blocks](docs/mechanisms/code-blocks.md)                                       |
| Display width     | The columns a string takes on screen; emoji and CJK take two.                                                                   | [Display width](docs/mechanisms/display-width.md)                                   |
| Document          | A Markdown file that belongs to the repository.                                                                                 | [Document discovery](docs/mechanisms/document-discovery.md)                         |
| Exclusion         | A path, path-except or rule that keeps a formatter from a document, or drops a linter's issue.                                  | [Configuration: Exclusions](docs/configuration.md#exclusions)                       |
| Feature           | A part of the `markright-core` crate a build can leave out, such as the disk or the threads, which the playground does without. | [Architecture: Crates](docs/architecture.md#crates)                                 |
| Formatter         | A rule that rewrites lines; `fmt` applies it and `check` reports what it would change.                                          | [Rules: Formatters](docs/rules.md#formatters)                                       |
| Index document    | The document that must link every other one, `CONTENTS.md` by default.                                                          | [Configuration](docs/configuration.md)                                              |
| Issue             | One problem at one line, named by the formatter or linter that found it.                                                        | [Rules: Commands](docs/rules.md#commands)                                           |
| Lazy continuation | A line without `>` that Markdown still joins to the paragraph of the blockquote above it.                                       | [Rules: Linters](docs/rules.md#linters)                                             |
| Linter            | A rule that reports issues and changes nothing.                                                                                 | [Rules: Linters](docs/rules.md#linters)                                             |
| Output format     | Where and how `check` writes issues: text, Checkstyle or SARIF.                                                                 | [Configuration: Output](docs/configuration.md#output)                               |
| Playground        | The page of the documentation site that runs markright, compiled to WebAssembly, on a document in the browser.                  | [Architecture: The documentation site](docs/architecture.md#the-documentation-site) |
| Protected span    | Part of a line the formatters leave as written: inline code, inline HTML, link targets.                                         | [Protected spans](docs/mechanisms/protected-spans.md)                               |
| Reference link    | A link written `[text][label]`, whose target is in a definition, `[label]: target`, elsewhere in the document.                  | [Rules: Linters](docs/rules.md#linters)                                             |
| Registry          | The list of every formatter, or every linter, that the runner, the commands and the configuration follow.                       | [Architecture: Registries](docs/architecture.md#registries)                         |
| Separator row     | The second row of a table, of dashes and colons, which also sets each column's alignment.                                       | [Rules: Formatters](docs/rules.md#formatters)                                       |
