# Rules

`markright` has formatters, which rewrite lines, and linters, which only report. Every one has a name; an issue prints
it, and [the configuration](configuration.md) turns it on or off.

Where CommonMark decides how Markdown is read, every rule follows it. Where it leaves a choice, such as which of `-`,
`*` and `+` marks a list item, a formatter applies the most common convention, so documents look alike across
repositories.

## Commands

| Command                   | What it does                                                                                                        | Exit code                  |
|---------------------------|---------------------------------------------------------------------------------------------------------------------|----------------------------|
| `markright check`         | reports every linter's issues and every line a formatter would change, in the configured formats; no writes         | `1` when any issue         |
| `markright check --fix`   | applies the formatters in place, lists the files it changed on standard error, then reports what no formatter fixes | `1` when any issue is left |
| `markright fmt`           | applies the formatters in place and lists the files it changed                                                      | `0`                        |
| `markright linters`       | lists the linters and whether the configuration enables each                                                        | `0`                        |
| `markright formatters`    | lists the formatters and whether the configuration enables each                                                     | `0`                        |
| `markright config verify` | checks the configuration file and changes nothing                                                                   | `0`                        |
| `markright version`       | prints the version; `--version` does the same                                                                       | `0`                        |
| `markright help`          | prints the commands and options; `--help` does the same, and `markright help check` explains one command            | `0`                        |

`check` and `fmt` take the documents under the current directory, or only the documents and directories given after
them, such as `markright check README.md docs`; run from the root, `markright check` and `markright check .` take
every document. Paths are relative to the current directory and must lie in the repository. Every document is still
read, so links and anchors into the others resolve, but only the ones given are reported and rewritten. A file that is
not one of the repository's documents, such as one Git ignores, stops the command.

The root of the repository is the nearest directory, from the current one up, that holds a configuration file or
`.git`; `--root` names it instead, and then `check` and `fmt` without paths take the whole of it. `--config` names
another configuration file. A configuration or repository that cannot be read, a path that cannot be taken, or a run
interrupted with Ctrl+C, ends any command with exit code `2`.

## Formatters

<!-- markright:formatters -->

| Formatter              | What it does                                                                                |
|------------------------|---------------------------------------------------------------------------------------------|
| `hard-tabs`            | turns tabs outside code into spaces, up to the next multiple of four columns                |
| `trailing-whitespace`  | removes spaces and tabs at the end of a line                                                |
| `heading-spacing`      | writes a heading as its `#`, one space and its text, without indentation or closing `#`     |
| `blockquote-space`     | puts a space between the `>` of a blockquote and its text                                   |
| `list-marker-space`    | leaves one space between the marker and the text of a list item                             |
| `arrows`               | `->`, `<-` and `<->` become `→`, `←` and `↔`, with a space on each side                     |
| `list-marker`          | marks the items of an unordered list with `-`, or the marker its settings choose            |
| `ordered-list-numbers` | numbers the items of an ordered list one after the other, or alike when its settings say so |
| `emphasis-marker`      | writes emphasis and strong emphasis with `*`, or the character its settings choose          |
| `link-spacing`         | removes the spaces just inside the brackets of a link's text                                |
| `code-span-spacing`    | removes the space just inside each backtick of a code span, which renders as nothing        |
| `bare-urls`            | puts a URL written as plain text in angle brackets, which makes it a link in any renderer   |
| `fence-marker`         | writes the fences of code blocks with backticks, or the character its settings choose       |
| `horizontal-rule`      | writes a horizontal rule as `---`, or the style its settings choose                         |
| `table-pipes`          | starts and ends every row of a table with a pipe                                            |
| `tables`               | pads every column of a table to one width                                                   |
| `heading-style`        | writes a heading underlined with `===` or `---` with `#` instead, on one line               |
| `block-spacing`        | puts a blank line before and after every heading, list, fenced code block and table         |
| `blank-lines`          | leaves at most one blank line in a row outside code                                         |
| `final-newline`        | ends the file with exactly one newline                                                      |

<!-- /markright:formatters -->

They run in that order, on prose only: code blocks and [protected spans](mechanisms/protected-spans.md) stay as
written. `hard-tabs` runs first, so the others see spaces; `heading-style`, `block-spacing`, `blank-lines` and
`final-newline` remove or add lines, so they run last. `check` reports what each formatter would change against the
file as it is, so every line number points into the file. A formatter changes how a document is written, never what
it renders to, except where its description says so: `arrows` turns arrows into their characters, `bare-urls` makes a
URL a link, `link-spacing` drops spaces at the edges of a link's text, and `trailing-whitespace` drops a line break
written as two spaces.

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
- `hard-tabs` counts columns from the start of the line, as an editor shows them, and leaves tabs in code, inline
  code included.
- `heading-spacing`, `heading-style` and `block-spacing` change only headings and blocks at the top of a document,
  outside lists and blockquotes, where moving a line could move it into or out of its container. `heading-spacing`
  keeps a `#` that ends the text, as in `C#`.
- `list-marker-space` changes only items of one line, and leaves five spaces or more, which start a code block
  inside the item. `ordered-list-numbers` keeps the first number, where the list starts, and leaves an item of more
  than one line whose new number would be longer or shorter, since its text would move from under its other lines.
  `formatters.settings.ordered-list-numbers.numbering` chooses `increasing`, the default, or `same`.
- `emphasis-marker` writes `*` and `**`, or `_` and `__` when its setting chooses them; emphasis that touches a letter
  or a digit keeps `*`, since `_` does not mark emphasis inside a word.
- `fence-marker` makes a fence longer when the code holds a line that the new fence would otherwise close on, and
  leaves a block that is never closed, or a backtick fence whose info string would hold a backtick.
- `horizontal-rule` leaves a rule right under a line of text, or on the first line, as written when its style is
  `---`: there `---` would underline the text into a heading, or open front matter.
- `code-span-spacing` removes a space only when the code has one on each side, which a renderer drops anyway; a
  space on one side is part of the code, as in ` 9.`, and stays. Code that starts or ends with a backtick keeps
  its spaces, as in `` ` `` for a backtick, and so does a span written over more than one line.
- `table-pipes` runs before `tables`, which aligns only rows that start with `|`, and leaves a table in a blockquote
  alone. A `|` escaped as `\|` at the end of a row is text, so the row still gets one.
- `bare-urls` finds a URL as GFM does, without the punctuation that ends its sentence, and leaves URLs in links, code
  and HTML alone.

## Linters

<!-- markright:linters -->

| Linter                     | What it enforces                                                                                    |
|----------------------------|-----------------------------------------------------------------------------------------------------|
| `links`                    | a relative link points to a file or directory inside the repository                                 |
| `anchors`                  | a `#fragment` matches a heading of the target document                                              |
| `unused-definitions`       | every reference definition is used by a reference link                                              |
| `duplicate-definitions`    | a reference label is defined once; a renderer ignores every definition after the first              |
| `reversed-links`           | a link is written `[text](target)`, not with its brackets swapped                                   |
| `link-text`                | a link's text says where it leads, not one of the phrases its settings list, such as `here`         |
| `alerts`                   | an alert tag is one of the allowed types and is followed by an empty `>` line                       |
| `blockquotes`              | a line that continues a blockquote's paragraph starts with `>`                                      |
| `blank-line-in-blockquote` | a blank line does not split a blockquote in two; a `>` alone keeps its paragraphs together          |
| `fences`                   | every code block is closed                                                                          |
| `fence-language`           | a fenced code block names its language                                                              |
| `code-block-style`         | a code block is fenced, so it can name its language, not indented                                   |
| `first-heading`            | a document starts with a top-level heading, after any front matter and HTML                         |
| `single-title`             | a document has one top-level heading, its title                                                     |
| `heading-increment`        | a heading is at most one level deeper than the heading before it                                    |
| `heading-without-space`    | a line meant as a heading has a space after its `#`                                                 |
| `duplicate-headings`       | no two headings of one section say the same, which would leave their anchors apart by a number only |
| `heading-punctuation`      | a heading does not end with `.`, `,`, `;`, `:` or `!`                                               |
| `emphasis-as-heading`      | a paragraph of emphasis alone is not used in place of a heading                                     |
| `list-indent`              | the items of one list are indented alike                                                            |
| `image-alt`                | an image has alternative text                                                                       |
| `table-columns`            | a table row has no more cells than the header                                                       |
| `line-length`              | a line is at most the configured number of characters                                               |
| `index`                    | the index document links every Markdown document of the repository                                  |
| `readme-links`             | the README links each of the configured documents that exists                                       |

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
- `first-heading` lets front matter and HTML, such as badges, come before the title. `single-title` reports every
  top-level heading after the first.
- `heading-punctuation` lets a heading end with `?`, as a question that names a section of answers.
- `list-indent` compares each item with the first of its list, and is satisfied when either the marker or the text
  starts at the same column, so numbers aligned on the right, ` 9.` over `10.`, are fine.
- `duplicate-definitions` reports every definition of a label after the first, which a renderer ignores, so a link
  through that label leads where the first one points.
- `duplicate-headings` compares headings of one section: those with the same nearest heading above them at a lower
  level. Two `## Setup` under different chapters are fine.
- `heading-without-space` reports a line that starts with `#` and no space, which CommonMark reads as text.
- `emphasis-as-heading` reports a paragraph of one line that is all emphasis and does not end with punctuation.
- `reversed-links` reports `(text)[target]`, unless `[target]` is a defined reference or the text follows a link.
- `blank-line-in-blockquote` lets an alert start a blockquote of its own, since two alerts in a row are two quotes.
- `code-block-style` reports indented code blocks, which cannot name their language.
- `link-text` compares a link's text, without case and the punctuation around it, with
  `linters.settings.link-text.phrases`.
- `index` and `readme-links` look at the documents their settings name; when that document is missing, they do not
  run. `index` does not ask for documents the linters exclude by path. Which documents exist is decided by
  [document discovery](mechanisms/document-discovery.md).
