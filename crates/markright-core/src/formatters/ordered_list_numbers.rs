//! The ordered-list-numbers formatter: the items of an ordered list numbered one after the other, or all alike.

use std::collections::HashMap;

use super::settings::Numbering;
use super::{About, Fix, Formatter, Settings, by_line, texts};
use crate::FIRST_RELEASE;
use crate::markdown::{Document, ItemMarker};

pub(super) struct OrderedListNumbers;

impl Formatter for OrderedListNumbers {
    fn about(&self) -> About {
        About {
            name: "ordered-list-numbers",
            description: "numbers the items of an ordered list one after the other, or alike when its settings say so",
            message: "list item numbered out of order",
            since: FIRST_RELEASE,
            example: "1. Build\n1. Test\n4. Release\n",
        }
    }

    /// The first item keeps its number, which is where the list starts. An item whose new number has more or fewer
    /// digits keeps its own when it runs over more than one line, since its text would move from under its other
    /// lines.
    fn apply(&self, document: &Document, settings: &Settings) -> Fix {
        let numbering = settings.ordered_list_numbers.numbering;
        let mut out = texts(document);
        // Each list's first number and how many of its items came before, by list: a nested list interrupts its
        // parent's items without ending them.
        let mut lists: HashMap<usize, (u64, u64)> = HashMap::new();
        for item in &document.items {
            let ItemMarker::Number { value, digits, .. } = item.marker else {
                continue;
            };
            let (first, before) = lists.entry(item.list).or_insert((value, 0));
            let expected = match numbering {
                Numbering::Increasing => *first + *before,
                Numbering::Same => *first,
            };
            *before += 1;
            let written = expected.to_string();
            let index = item.line - 1;
            let keeps_width = written.len() == digits || item.last_line == item.line;
            if expected != value && keeps_width && !document.lines[index].code {
                out[index].replace_range(item.column..item.column + digits, &written);
            }
        }
        by_line(&document.lines, out)
    }
}
