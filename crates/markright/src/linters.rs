//! The registry of linters: rules that report issues in a document or in the whole repository.

// What the linters share.
mod repo;
mod rule;
pub mod settings;
mod targets;

// The linters, one per file.
mod alerts;
mod anchors;
mod blockquotes;
mod fence_language;
mod fences;
mod heading_increment;
mod image_alt;
mod index;
mod line_length;
mod links;
mod readme_links;
mod table_columns;
mod unused_definitions;

pub use repo::{Context, Repo};
pub use rule::{About, DocumentLinter, Example, Linter, RepositoryLinter};
pub use settings::Settings;

use targets::{escapes, targets};

/// The index the index linter checks and the readme-links linter expects the README to link, unless the
/// configuration names others.
const INDEX_DOCUMENT: &str = "CONTENTS.md";
/// A document the examples of several linters link: its path and its content.
const EXAMPLE_GUIDE: (&str, &str) = ("docs/guide.md", "# Guide\n\n## Install\n");

/// Every linter, in the order they run.
pub static ALL: &[Linter] = &[
	Linter::Document(&links::Links),
	Linter::Document(&anchors::Anchors),
	Linter::Document(&unused_definitions::UnusedDefinitions),
	Linter::Document(&alerts::Alerts),
	Linter::Document(&blockquotes::Blockquotes),
	Linter::Document(&fences::Fences),
	Linter::Document(&fence_language::FenceLanguage),
	Linter::Document(&heading_increment::HeadingIncrement),
	Linter::Document(&image_alt::ImageAlt),
	Linter::Document(&table_columns::TableColumns),
	Linter::Document(&line_length::LineLength),
	Linter::Repository(&index::Index),
	Linter::Repository(&readme_links::ReadmeLinks),
];

/// The name of every linter, in the order they run.
#[must_use]
pub fn names() -> Vec<&'static str> {
	ALL.iter().map(|l| l.about().name).collect()
}

/// The linter called `name`.
#[must_use]
pub fn find(name: &str) -> Option<Linter> {
	ALL.iter().copied().find(|l| l.about().name == name)
}

#[cfg(test)]
mod tests;
