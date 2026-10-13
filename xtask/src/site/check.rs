//! The check of a built site: every link inside the site must lead to a file of it and, when it names one, to an
//! anchor of the page. Zola checks the links of the pages' Markdown; this also covers the templates'.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

static LINK: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r#"(?:href|src)="?([^"\s>]+)"#).expect("the link pattern is valid"));
static ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\sid="?([^"\s>]+)"#).expect("the id pattern is valid"));
static SCHEME: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^(?:[A-Za-z][A-Za-z0-9+.-]*:|//)").expect("the scheme pattern is valid"));

/// What the check needs of one page: the anchors it defines and the links it holds.
struct Page {
	ids: HashSet<String>,
	links: Vec<String>,
}

/// Fails when a link of the site built into `public`, served at `base`, an address such as `https://host/path/`, leads
/// to no file or no anchor, and prints every such link.
pub(crate) fn check(public: &Path, base: &str) -> io::Result<()> {
	let broken = broken(public, base)?;
	for problem in &broken {
		eprintln!("{problem}");
	}
	if broken.is_empty() {
		println!("every link of the site resolves");
		Ok(())
	} else {
		Err(io::Error::other(format!("{} broken link(s)", broken.len())))
	}
}

/// Every link of the site that leads nowhere, as `page: problem`. A link to `base` is a link inside the site, by the
/// path `base` is served under.
pub(super) fn broken(public: &Path, base: &str) -> io::Result<Vec<String>> {
	let host_and_path = base.split_once("://").map_or(base, |(_, rest)| rest);
	let path = host_and_path.find('/').map_or("/", |start| &host_and_path[start..]);
	let mut pages = BTreeMap::new();
	collect(public, "", &mut pages)?;
	let mut broken = Vec::new();
	for (name, page) in &pages {
		for link in &page.links {
			let link = link
				.strip_prefix(base)
				.map_or_else(|| link.clone(), |rest| format!("{path}{rest}"));
			if let Some(problem) = problem(public, path, &pages, name, &link) {
				broken.push(format!("{name}: {problem}"));
			}
		}
	}
	Ok(broken)
}

/// Reads every page under `directory`, which is `prefix` in the site, into `pages`, by its path in the site.
fn collect(directory: &Path, prefix: &str, pages: &mut BTreeMap<String, Page>) -> io::Result<()> {
	for entry in fs::read_dir(directory)? {
		let entry = entry?;
		let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
		if entry.file_type()?.is_dir() {
			collect(&entry.path(), &format!("{name}/"), pages)?;
		} else if Path::new(&name)
			.extension()
			.is_some_and(|extension| extension == "html")
		{
			let html = fs::read_to_string(entry.path())?;
			let page = Page {
				ids: ID.captures_iter(&html).map(|id| id[1].to_owned()).collect(),
				links: LINK.captures_iter(&html).map(|link| link[1].to_owned()).collect(),
			};
			pages.insert(name, page);
		}
	}
	Ok(())
}

/// What is wrong with `link` on the page `from`, or nothing when it leads to a file of the site and, when it names
/// one, to an anchor of the page. Links to other sites are not checked.
fn problem(public: &Path, base: &str, pages: &BTreeMap<String, Page>, from: &str, link: &str) -> Option<String> {
	if SCHEME.is_match(link) {
		return None;
	}
	let (path, fragment) = link
		.split_once('#')
		.map_or((link, None), |(path, fragment)| (path, Some(fragment)));
	let path = path.split('?').next().unwrap_or_default();
	let Some(mut target) = site_path(base, from, path) else {
		return Some(format!("leaves the site, which is served under {base}: {link}"));
	};
	if !pages.contains_key(&target) {
		let file = public.join(&target);
		if file.is_file() {
			// A file that is not a page, such as an image or a stylesheet.
			return None;
		}
		if !file.is_dir() {
			return Some(format!("no page for {link}"));
		}
		target = format!("{target}/index.html");
	}
	match (pages.get(&target), fragment) {
		(None, _) => Some(format!("no page for {link}")),
		(Some(page), Some(fragment)) if !fragment.is_empty() && !page.ids.contains(fragment) => {
			Some(format!("no anchor for {link}"))
		}
		_ => None,
	}
}

/// The file in the site that `path`, a link on the page `from`, leads to. A path from the server's root must start
/// with `base`; a relative one is resolved against the page's directory. Nothing for a path that leaves the site.
fn site_path(base: &str, from: &str, path: &str) -> Option<String> {
	let inside = if path.is_empty() {
		return Some(from.to_owned());
	} else if path.starts_with('/') {
		path.strip_prefix(base)?.to_owned()
	} else {
		let directory = from.rsplit_once('/').map_or("", |(directory, _)| directory);
		format!("{directory}/{path}")
	};
	let mut parts: Vec<&str> = Vec::new();
	for part in inside.split('/') {
		match part {
			"" | "." => {}
			".." => {
				parts.pop()?;
			}
			_ => parts.push(part),
		}
	}
	let joined = parts.join("/");
	if joined.is_empty() || inside.ends_with('/') {
		Some(if joined.is_empty() {
			"index.html".to_owned()
		} else {
			format!("{joined}/index.html")
		})
	} else {
		Some(joined)
	}
}
