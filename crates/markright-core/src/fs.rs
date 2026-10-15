//! The files of a repository as the checks read them: on disk for the command, in memory for the playground.

use std::collections::BTreeMap;
use std::io;
use std::ops::Bound;
use std::path::Path;

/// Whether the file at `path` is a Markdown document: whether its name ends in `.md`.
#[must_use]
pub(crate) fn is_markdown(path: &str) -> bool {
    Path::new(path).extension().is_some_and(|extension| extension == "md")
}

/// Read access to the files of a repository, by slash-separated path relative to its root.
pub(crate) trait Files: Sync {
    /// The content of the file at `path`.
    ///
    /// # Errors
    ///
    /// Returns why the file could not be read.
    fn read(&self, path: &str) -> io::Result<Vec<u8>>;

    /// Whether `path` is a file or a directory.
    ///
    /// # Errors
    ///
    /// Returns why the answer is unknown: not that the path is missing, which is `Ok(false)`.
    fn exists(&self, path: &str) -> io::Result<bool>;
}

/// A repository held in memory: file contents by path. Directories are implied by the paths of the files under them.
impl Files for BTreeMap<String, String> {
    fn read(&self, path: &str) -> io::Result<Vec<u8>> {
        self.get(path)
            .map(|content| content.clone().into_bytes())
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn exists(&self, path: &str) -> io::Result<bool> {
        if path == "." || self.contains_key(path) {
            return Ok(true);
        }
        let dir = format!("{path}/");
        Ok(self
            .range::<str, _>((Bound::Included(dir.as_str()), Bound::Unbounded))
            .next()
            .is_some_and(|(name, _)| name.starts_with(&dir)))
    }
}

#[cfg(test)]
mod tests {
    use super::Files;
    use std::collections::BTreeMap;

    #[test]
    fn memory_has_files_and_the_directories_above_them() {
        let files = BTreeMap::from([("docs/guide/a.md".to_owned(), "# A\n".to_owned())]);
        for (path, want) in [
            ("docs/guide/a.md", true),
            ("docs/guide", true),
            ("docs", true),
            (".", true),
            ("doc", false),
        ] {
            assert_eq!(files.exists(path).unwrap(), want, "{path}");
        }
        assert_eq!(files.read("docs/guide/a.md").unwrap(), b"# A\n");
        assert_eq!(files.read("docs").unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
}
