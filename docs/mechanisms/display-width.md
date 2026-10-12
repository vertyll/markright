# Display width

`tables` pads cells to the width they take on screen, so a column lines up in an editor and in a terminal:

| Characters                                                   | Width |
|--------------------------------------------------------------|-------|
| most characters, Polish letters and arrows included          | 1     |
| emoji with an emoji presentation, such as ✅ and ❌          | 2     |
| a character followed by U+FE0F (emoji presentation), like ⚠️ | 2     |
| CJK ideographs, Hangul and full-width forms                  | 2     |
| combining marks, variation selectors, the zero-width joiner  | 0     |

The wide ranges follow Unicode's East Asian Width and emoji presentation data; a joined emoji sequence counts each of
its emoji. `line-length` counts characters, not width.
