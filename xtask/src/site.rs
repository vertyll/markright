//! The content of the documentation site, which Zola builds in `site/`: the repository's documents at their places on
//! the site, and a page for every formatter and linter with its example as markright checks it. Also the check of the
//! links of the built site.

mod check;
mod diagrams;
mod documents;
mod markup;
mod rules;

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

pub(crate) use check::check;

/// The directory Zola reads the pages from. Every file in it is written here, so it is emptied first.
const CONTENT: &str = "site/content";

/// The pages whose content is all in their templates: their path in the content and their front matter.
const TEMPLATE_PAGES: [(&str, &str, &str); 3] = [
	("_index.md", "markright", "index.html"),
	("playground.md", "Playground", "playground.html"),
	("settings.md", "Settings", "settings.html"),
];

/// Writes the content of the site.
pub(crate) fn generate(root: &Path) -> io::Result<()> {
	let mut pages = documents::pages(root)?;
	pages.extend(rules::pages(root)?);
	for (path, title, template) in TEMPLATE_PAGES {
		let front = markup::front_matter(
			&[("title", markup::string(title)), ("template", markup::string(template))],
			&[],
		);
		pages.insert(path.to_owned(), front);
	}
	write(&root.join(CONTENT), &pages)?;
	println!("wrote {} pages into {CONTENT}", pages.len());
	Ok(())
}

/// Empties `content` and writes `pages` into it, each at its path.
fn write(content: &Path, pages: &BTreeMap<String, String>) -> io::Result<()> {
	if content.exists() {
		fs::remove_dir_all(content)?;
	}
	for (path, text) in pages {
		let file = content.join(path);
		if let Some(directory) = file.parent() {
			fs::create_dir_all(directory)?;
		}
		fs::write(file, text)?;
	}
	Ok(())
}

#[cfg(test)]
mod tests;
