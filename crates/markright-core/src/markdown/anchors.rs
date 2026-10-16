//! The anchors a document defines: GitHub's slug of each heading, and the ids of HTML elements.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use unicode_properties::{GeneralCategory, GeneralCategoryGroup, UnicodeGeneralCategory};

static HTML_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<[A-Za-z][^>]*?[\t\n\f\r ](?:id|name)[\t\n\f\r ]*=[\t\n\f\r ]*["']([^"']+)["']"#)
        .expect("the HTML id pattern is valid")
});

/// Collects anchors into a set, numbering a heading's slug when an earlier heading had it.
#[derive(Default)]
pub(super) struct Anchors {
    seen: HashMap<String, usize>,
    set: HashSet<String>,
}

impl Anchors {
    /// The anchors collected.
    pub(super) fn into_set(self) -> HashSet<String> {
        self.set
    }

    /// Adds the anchor of a heading whose text, without markup, is `text`, and returns it.
    pub(super) fn heading(&mut self, text: &str) -> String {
        let slug = slugify(text);
        let count = *self
            .seen
            .entry(slug.clone())
            .and_modify(|count| *count += 1)
            .or_insert(0);
        let anchor = if count == 0 { slug } else { format!("{slug}-{count}") };
        self.set.insert(anchor.clone());
        anchor
    }

    /// Adds the id or name of every HTML element in `html`.
    pub(super) fn html(&mut self, html: &str) {
        for captures in HTML_ID.captures_iter(html) {
            self.set.insert(captures[1].to_lowercase());
        }
    }
}

/// The anchor GitHub derives from a heading's text: lowercased, with its word characters and `-` kept, spaces turned
/// into `-`, and everything else dropped. The suffix of a repeated heading is not added.
pub(super) fn slugify(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' => Some(c),
            _ if is_word(c) => Some(c),
            _ => None,
        })
        .collect()
}

/// A word character as GitHub's slugs keep it, Unicode's `\p{Word}`: a letter or anything alphabetic, a mark such as a
/// combining accent, a decimal digit, or connector punctuation such as `_`. A digit of another kind, as in `²` or
/// `½`, is dropped.
fn is_word(c: char) -> bool {
    c.is_alphabetic()
        || c.general_category_group() == GeneralCategoryGroup::Mark
        || matches!(
            c.general_category(),
            GeneralCategory::DecimalNumber | GeneralCategory::ConnectorPunctuation
        )
}
