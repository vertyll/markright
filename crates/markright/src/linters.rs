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

pub(crate) use repo::{Context, Repo};
pub use rule::{About, Example, Linter};
pub(crate) use rule::{DocumentLinter, Kind, RepositoryLinter};
pub use settings::Settings;

use targets::{escapes, targets};

/// The index the index linter checks and the readme-links linter expects the README to link, unless the
/// configuration names others.
const INDEX_DOCUMENT: &str = "CONTENTS.md";
/// A document the examples of several linters link: its path and its content.
const EXAMPLE_GUIDE: (&str, &str) = ("docs/guide.md", "# Guide\n\n## Install\n");

/// Every linter, in the order they run.
pub static ALL: &[Linter] = &[
	Linter::document(&links::Links),
	Linter::document(&anchors::Anchors),
	Linter::document(&unused_definitions::UnusedDefinitions),
	Linter::document(&alerts::Alerts),
	Linter::document(&blockquotes::Blockquotes),
	Linter::document(&fences::Fences),
	Linter::document(&fence_language::FenceLanguage),
	Linter::document(&heading_increment::HeadingIncrement),
	Linter::document(&image_alt::ImageAlt),
	Linter::document(&table_columns::TableColumns),
	Linter::document(&line_length::LineLength),
	Linter::repository(&index::Index),
	Linter::repository(&readme_links::ReadmeLinks),
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
