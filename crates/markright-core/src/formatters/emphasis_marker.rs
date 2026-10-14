//! The emphasis-marker formatter: emphasis and strong emphasis written with one character, `*` or `_`.

use super::{About, Fix, Formatter, Settings, by_line, texts};
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

	/// `_` does not mark emphasis inside a word, so emphasis that touches a letter or a digit keeps `*`.
	fn apply(&self, document: &Document, settings: &Settings) -> Fix {
		let marker = settings.emphasis_marker.marker.as_char();
		let mut out = texts(document);
		for emphasis in &document.emphasis {
			let written = |span: &Span| out[span.line - 1][span.columns.clone()].chars().next();
			if written(&emphasis.open) == Some(marker) {
				continue;
			}
			let code = |span: &Span| document.lines[span.line - 1].code;
			let inside_word = marker == '_'
				&& (touches_word(&out, &emphasis.open, true) || touches_word(&out, &emphasis.close, false));
			if code(&emphasis.open) || code(&emphasis.close) || inside_word {
				continue;
			}
			for span in [&emphasis.open, &emphasis.close] {
				let run = marker.to_string().repeat(span.columns.len());
				out[span.line - 1].replace_range(span.columns.clone(), &run);
			}
		}
		by_line(&document.lines, out)
	}
}

/// Whether a letter or a digit stands right before the opening delimiter at `span`, or right after the closing one.
fn touches_word(lines: &[String], span: &Span, opening: bool) -> bool {
	let line = &lines[span.line - 1];
	let neighbor = if opening {
		line[..span.columns.start].chars().next_back()
	} else {
		line[span.columns.end..].chars().next()
	};
	neighbor.is_some_and(char::is_alphanumeric)
}
