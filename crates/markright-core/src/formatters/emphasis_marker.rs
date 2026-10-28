//! The emphasis-marker formatter: emphasis and strong emphasis written with one character, `*` or `_`.

use super::{About, Edit, Fix, Formatter, Replacement, Settings, keeping_meaning};
use crate::FIRST_RELEASE;
use crate::markdown::{Document, Span};

pub(super) struct EmphasisMarker;

impl Formatter for EmphasisMarker {
    fn about(&self) -> About {
        About {
            name: "emphasis-marker",
            description: "writes emphasis and strong emphasis with `*`, or the character its settings choose",
            message: "emphasis written differently from the configured character",
            since: FIRST_RELEASE,
            example: "Run it _once_, then __never again__.\n",
        }
    }

    /// `_` does not mark emphasis inside a word, so emphasis that touches a letter or a digit keeps `*`. A run of
    /// delimiters that would join another once rewritten, as `*_foo_*` would become strong emphasis, keeps its own.
    fn apply(&self, document: &Document, settings: &Settings) -> Fix {
        let marker = settings.emphasis_marker.marker.as_char();
        let lines: Vec<&str> = document.lines.iter().map(|line| line.text.as_str()).collect();
        let edits: Vec<Edit> = document
            .emphasis
            .iter()
            .filter(|emphasis| {
                let written = lines[emphasis.open.line - 1][emphasis.open.columns.clone()]
                    .chars()
                    .next();
                let code = |span: &Span| document.lines[span.line - 1].code;
                let inside_word = marker == '_'
                    && (touches_word(&lines, &emphasis.open, true) || touches_word(&lines, &emphasis.close, false));
                written != Some(marker) && !code(&emphasis.open) && !code(&emphasis.close) && !inside_word
            })
            .map(|emphasis| {
                [&emphasis.open, &emphasis.close]
                    .into_iter()
                    .map(|span| Replacement {
                        line: span.line - 1,
                        bytes: span.columns.clone(),
                        text: marker.to_string().repeat(span.columns.len()),
                    })
                    .collect()
            })
            .collect();
        keeping_meaning(document, &edits)
    }
}

/// Whether a letter or a digit stands right before the opening delimiter at `span`, or right after the closing one.
fn touches_word(lines: &[&str], span: &Span, opening: bool) -> bool {
    let line = lines[span.line - 1];
    let neighbor = if opening {
        line[..span.columns.start].chars().next_back()
    } else {
        line[span.columns.end..].chars().next()
    };
    neighbor.is_some_and(char::is_alphanumeric)
}
