use super::{About, Context, DocumentLinter, EXAMPLE_GUIDE, Example, V0_1_0, escapes, targets};
use crate::Error;
use crate::issue::Finding;
use crate::markdown::Document;

pub(super) struct Links;

impl DocumentLinter for Links {
	fn about(&self) -> About {
		About {
			name: "links",
			description: "a relative link points to a file or directory inside the repository",
			since: V0_1_0,
			example: Example::new("See the [guide](docs/gide.md).\n", "See the [guide](docs/guide.md).\n")
				.with_files(&[EXAMPLE_GUIDE]),
		}
	}

	fn check(&self, c: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		let mut out = Vec::new();
		for target in targets(path, doc) {
			let Some(file) = &target.file else { continue };
			if escapes(file) {
				out.push(Finding::new(
					path,
					target.line,
					format!("link leaves the repository: {}", target.raw),
				));
			} else if !c.repo.exists(file).map_err(|source| Error::Resolve {
				linter: "links",
				path: file.clone(),
				source,
			})? {
				out.push(Finding::new(path, target.line, format!("broken link: {}", target.raw)));
			}
		}
		for link in &doc.links {
			if link.target.is_empty() {
				out.push(Finding::new(
					path,
					link.line,
					format!("link without a target: [{}]()", link.text),
				));
			} else if !link.image && !link.has_content {
				out.push(Finding::new(
					path,
					link.line,
					format!("link without text: []({})", link.target),
				));
			}
		}
		for reference in &doc.undefined_references {
			out.push(Finding::new(
				path,
				reference.line,
				format!("undefined reference: [{}]", reference.label),
			));
		}
		Ok(out)
	}
}
