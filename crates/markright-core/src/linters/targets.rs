//! Where the links of a document lead: the file of the repository each one resolves to, and the fragment it names.

use std::borrow::Cow;

use percent_encoding::percent_decode_str;

use crate::markdown::{Document, is_external};

/// A link of a document to a place in the repository: the file it resolves to, unless it points into the document
/// itself, and the fragment it names, if any.
pub(super) struct Target<'a> {
    /// The line the link is written on.
    pub(super) line: usize,
    /// The target as written.
    pub(super) raw: &'a str,
    /// The file it resolves to, relative to the root; none for a link into the document itself.
    pub(super) file: Option<String>,
    /// The fragment it names, percent escapes decoded.
    pub(super) anchor: Option<String>,
}

impl Target<'_> {
    /// The document the target's fragment is looked up in: the file it names, or the linking document itself.
    pub(super) fn document<'p>(&'p self, from: &'p str) -> &'p str {
        self.file.as_deref().unwrap_or(from)
    }
}

/// The links of the document at `path` that stay in the repository, resolved against its directory.
pub(super) fn targets<'d>(path: &str, document: &'d Document) -> Vec<Target<'d>> {
    document
        .links
        .iter()
        .filter(|link| !link.target.is_empty() && !is_external(&link.target))
        .map(|link| {
            let (file, anchor) = link
                .target
                .split_once('#')
                .map_or((link.target.as_str(), None), |(file, fragment)| (file, Some(fragment)));
            let file = file.split_once('?').map_or(file, |(path, _)| path);
            Target {
                line: link.line,
                raw: &link.target,
                file: (!file.is_empty()).then(|| join(parent(path), &unescape(file))),
                anchor: anchor.map(unescape),
            }
        })
        .collect()
}

/// Decodes the percent escapes of a link target, so `my%20file.md` names `my file.md`; a target whose escapes do not
/// make UTF-8 stays as written.
fn unescape(s: &str) -> String {
    percent_decode_str(s)
        .decode_utf8()
        .map_or_else(|_| s.to_owned(), Cow::into_owned)
}

/// The directory of a slash-separated path, `.` for one at the root.
pub(super) fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or(".", |(dir, _)| dir)
}

/// Joins `relative` to `dir` and resolves its `.` and `..` elements; a path that climbs above the root keeps its
/// leading `..`.
fn join(dir: &str, relative: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in dir.split('/').chain(relative.split('/')) {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|&last| last != "..") => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() {
        ".".to_owned()
    } else {
        parts.join("/")
    }
}

/// Whether a resolved path climbs out of the repository.
pub(super) fn escapes(resolved: &str) -> bool {
    resolved == ".." || resolved.starts_with("../")
}
