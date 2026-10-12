# Protected spans

The formatters rewrite prose only. On every line they leave alone:

- inline code, so `` `a -> b` `` stays as written, also when it runs onto the next line of the paragraph;
- inline HTML, such as an HTML comment within a paragraph, whose `<!--` and `-->` are not arrows;
- link targets and the targets of reference definitions, so a path or URL with `->` in it still resolves.

[Code blocks](code-blocks.md), fenced or indented, and HTML blocks, comments that span several lines among them, are
left alone as a whole, so Mermaid's `->>` and `-->` and any arrows in code keep their meaning. A link's text is prose
and is fixed.
