# Code blocks

No formatter or linter looks inside a code block or an HTML block, so the parser first decides which lines are code.

The document is parsed as CommonMark with the GFM tables, strikethrough and task lists, by
[pulldown-cmark](https://docs.rs/pulldown-cmark), which gives every block the bytes of the source it covers. The lines
it marks are exactly the ones a CommonMark renderer shows as code or passes through as HTML:

- **Fenced block:** opens with three or more backticks or tildes. It closes with a line of the same character, at
  least as long. A block that never closes runs to the end of its container, and `fences` reports it.
- **Indented block:** a line indented by four or more columns that does not continue a paragraph. Columns count from
  where the text of its list item or blockquote starts.
- **HTML block:** starts with a line such as `<div>`, `<p align="center">` or `<!--`. It runs to its closing tag or
  to the next blank line, as CommonMark defines for each kind. Its `id` and `name` attributes still define anchors.
- **Front matter:** YAML between two `---` lines, or TOML between two `+++` lines, starting on the first line of the
  document. It is data for the tools that publish the document, so no rule reads it as Markdown.

```markdown
1. An item.

   A paragraph of the item, not code.

       Code inside the item.

> ```bash
> code inside a blockquote
> ```
```

Fences, indentation and HTML blocks work the same inside lists and blockquotes, at any depth.
