# Rules

`markright` has formatters, which rewrite lines, and linters, which only report. Every one has a name; an issue prints
it, and [the configuration](configuration.md) turns it on or off.

Where CommonMark decides how Markdown is read, every rule follows it. Where it leaves a choice, such as which of `-`,
`*` and `+` marks a list item, a formatter applies the most common convention, so documents look alike across
repositories.

## Commands

| Command                   | What it does                                                                                                | Exit code          |
|---------------------------|-------------------------------------------------------------------------------------------------------------|--------------------|
| `markright check`         | reports every linter's issues and every line a formatter would change, in the configured formats; no writes | `1` when any issue |
| `markright fmt`           | applies the formatters in place and lists the files it changed                                              | `0`                |
| `markright linters`       | lists the linters and whether the configuration enables each                                                | `0`                |
| `markright formatters`    | lists the formatters and whether the configuration enables each                                             | `0`                |
| `markright config verify` | checks the configuration file and changes nothing                                                           | `0`                |
| `markright version`       | prints the version; `--version` does the same                                                               | `0`                |
| `markright help`          | prints the commands and options; `--help` does the same, and `markright help run` explains one command      | `0`                |

Each takes an optional root, the current directory by default, and `--config` with another configuration file. A
configuration or repository that cannot be read, or a run interrupted with Ctrl+C, ends any command with exit code
`2`.

## Formatters

<!-- markright:formatters -->

| Formatter             | What it does                                                                     |
|-----------------------|----------------------------------------------------------------------------------|
| `trailing-whitespace` | removes spaces and tabs at the end of a line                                     |
| `arrows`              | `->`, `<-` and `<->` become `→`, `←` and `↔`, with a space on each side          |
| `list-marker`         | marks the items of an unordered list with `-`, or the marker its settings choose |
| `tables`              | pads every column of a table to one width                                        |
| `blank-lines`         | leaves at most one blank line in a row outside code                              |
| `final-newline`       | ends the file with exactly one newline                                           |

<!-- /markright:formatters -->

They run in that order, on prose only: code blocks and [protected spans](mechanisms/protected-spans.md) stay as
written. `blank-lines` and `final-newline` remove or add lines, so they run last; `check` reports what each formatter
would change against the file as it is, so every line number points into the file.

- `arrows` turns any run of dashes ending in `>` or starting with `<` into one arrow, so `-->` becomes `→` as well.
- `tables` pads cells by [display width](mechanisms/display-width.md) and keeps each column's alignment: `:--` pads
  on the right, `--:` on the left, `:-:` on both sides. A block counts as a table only when its second row is a
  separator, so a paragraph that starts with `|` stays as it is.
- `list-marker` uses `-`, or `*` or `+` when `formatters.settings.list-marker.marker` chooses it, as
  [Configuration: Settings](configuration.md#settings) shows. It leaves a list alone when another list stands right
  before or after it: there, the different marker is what keeps the two lists apart, and changing it would merge
  them.
- `tables` leaves a table alone when a row has more cells than the header, since padding the header would add a
  column; `table-columns` reports it instead.
- `blank-lines` keeps blank lines inside code and HTML blocks, where they are content; elsewhere Markdown renders
  several blank lines as one.
- `final-newline` removes blank lines at the end of the file and makes it end with one newline.

## Linters

<!-- markright:linters -->

| Linter               | What it enforces                                                              |
|----------------------|-------------------------------------------------------------------------------|
| `links`              | a relative link points to a file or directory inside the repository           |
| `anchors`            | a `#fragment` matches a heading of the target document                        |
| `unused-definitions` | every reference definition is used by a reference link                        |
| `alerts`             | an alert tag is one of the allowed types and is followed by an empty `>` line |
| `blockquotes`        | a line that continues a blockquote's paragraph starts with `>`                |
| `fences`             | every code block is closed                                                    |
| `fence-language`     | a fenced code block names its language                                        |
| `heading-increment`  | a heading is at most one level deeper than the heading before it              |
| `image-alt`          | an image has alternative text                                                 |
| `table-columns`      | a table row has no more cells than the header                                 |
| `line-length`        | a line is at most the configured number of characters                         |
| `index`              | the index document links every Markdown document of the repository            |
| `readme-links`       | the README links each of the configured documents that exists                 |

<!-- /markright:linters -->

Linters see a document as it is after the formatters, and skip [code blocks](mechanisms/code-blocks.md).

- `links` and `anchors` resolve the target relative to the document, and a fragment against the target's headings:
  [Anchors](mechanisms/anchors.md). A link that climbs above the root is an issue even when the file exists, since it
  breaks wherever the repository is viewed on its own. A reference link, `[text][label]` or `[label][]`, is checked
  through its definition, `[label]: target`, and a label with no definition in the document is an issue, since it
  renders as plain text. A link with an empty target, `[text]()`, or with no content at all, `[](target)`, is an
  issue too. External links, `mailto:` and links inside inline code are not followed.
- A link is found wherever a paragraph has it, also when its text wraps onto the next line, and its issue is
  reported on the line its target starts on. Brackets inside the text and balanced parentheses inside the target
  belong to the link, a target in `<...>` may contain spaces, backslash escapes and entities in a target are resolved
  as CommonMark resolves them, and a target is percent-decoded and loses its `?query` before the file is looked up,
  so `my%20file.md` finds `my file.md`.
- A reference definition is what CommonMark takes for one: it starts a block, so a line that looks like a definition
  inside a paragraph is text, and when a label is defined twice the first definition wins and the second is ignored,
  as a renderer ignores it.
- `alerts` expects the form below; anything else in the tag, or text right after it, is an issue.

  ```markdown
  > [!WARNING]
  >
  > The text.
  ```

- `blockquotes` reports a lazy line: one without `>` that Markdown still joins to the paragraph of the quote above.
  A line that starts a block of its own, such as a heading or a list, ends the quote and is not reported.
- `fence-language` reports a fenced block with no info string; `text` names plain text, which renders without
  highlighting.
- `heading-increment` lets a heading go back up any number of levels, but down only one, so `##` after `####` is
  fine and `####` after `##` is not.
- `image-alt` reports an image whose alternative text is empty, also inside a link, as in a badge.
- `table-columns` reports a row with more cells than the header: the extra cells are not rendered. A row with fewer
  cells renders with empty ones, and `tables` pads it.
- `unused-definitions` reports a definition, `[label]: target`, that no reference link uses: `[text][label]`,
  `[label][]` or `[label]`.
- `line-length` counts characters, and leaves out table rows, badges, lines that start with HTML and lines holding a
  URL of 60 characters or more, none of which can be wrapped.
- `index` and `readme-links` look at the documents their settings name; when that document is missing, they do not
  run. `index` does not ask for documents the linters exclude by path. Which documents exist is decided by
  [document discovery](mechanisms/document-discovery.md).
