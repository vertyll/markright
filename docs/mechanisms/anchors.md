# Anchors

A link's `#fragment` must name an anchor of the document it points to; without a file, of the document it is in.
Every heading defines one, both `## Heading` and a line underlined with `===` or `---`, and so does an HTML element
with an `id`, or a `name`, which keeps its value as written. Each heading's anchor is derived the way GitHub derives
it:

1. The heading is reduced to the text it renders: a link keeps its text, an image and HTML tags disappear,
   emphasis loses its `*`, `_` and `~~` (an `_` inside a word, as in `snake_case`, stays), inline code keeps its
   text as written and loses the backticks, and an entity such as `&amp;` becomes the character it stands for.
2. The text is lowercased.
3. Word characters and `-` are kept, as Unicode's `\p{Word}` defines them: letters of any script, marks such as a
   combining accent, decimal digits and connector punctuation such as `_`. A space becomes `-`; everything else is
   dropped, other digits such as `²` or `½` included.
4. A heading whose anchor is already taken gets `-1`, the next `-2`, and so on.

| Heading                    | Anchor            |
|----------------------------|-------------------|
| `## Where do I look?`      | `where-do-i-look` |
| ``## The `lang` cookie``   | `the-lang-cookie` |
| `## Zażółć gęślą`          | `zażółć-gęślą`    |
| `## A → B`                 | `a--b`            |
| `## x² and ½ cup`          | `x-and--cup`      |
| a second `## Code`         | `code-1`          |
| `Setup` over `-----`       | `setup`           |
| `## See [the guide](a.md)` | `see-the-guide`   |
| `## **Bold** and _it_`     | `bold-and-it`     |

Headings and elements inside [code blocks](code-blocks.md) or inline code define nothing. A fragment is compared
lowercased, as GitHub compares it. Fragments of links to files that are not Markdown are not checked.
