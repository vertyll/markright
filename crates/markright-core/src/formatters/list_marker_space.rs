//! The list-marker-space formatter: one space between a list marker and the text of a one-line item.

use super::{About, Edit, Fix, Formatter, Replacement, Settings, keeping_meaning};
use crate::FIRST_RELEASE;
use crate::markdown::Document;

/// The most spaces after a marker that still start the item's text; five or more start an indented code block.
const MOST_SPACES: usize = 4;

pub(super) struct ListMarkerSpace;

impl Formatter for ListMarkerSpace {
    fn about(&self) -> About {
        About {
            name: "list-marker-space",
            description: "leaves one space between the marker and the text of a list item",
            message: "more than one space after a list marker",
            since: FIRST_RELEASE,
            example: "-   first\n-   second\n",
        }
    }

    /// Only items of one line change: moving the text of a longer one would move the column its other lines are
    /// measured from. A line below the item that the new column would take into it, though the old one left it out,
    /// keeps the item as it is.
    fn apply(&self, document: &Document, _: &Settings) -> Fix {
        let edits: Vec<Edit> = document
            .items
            .iter()
            .filter_map(|item| {
                let index = item.line - 1;
                let line = &document.lines[index];
                let after = item.column + item.marker.width();
                let one_line = item.last_line == item.line && !line.code;
                let text_follows = line.text.len() > after + item.spaces;
                (one_line && text_follows && (2..=MOST_SPACES).contains(&item.spaces)).then(|| {
                    vec![Replacement {
                        line: index,
                        bytes: after..after + item.spaces,
                        text: " ".to_owned(),
                    }]
                })
            })
            .collect();
        keeping_meaning(document, &edits)
    }
}
