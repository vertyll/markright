//! The SARIF 2.1.0 report. Its types are the parts of a SARIF log the report fills, named as the specification
//! names them.

use std::io::{self, Write};

use serde::Serialize;

use super::Rule;
use crate::issue::Issue;

const VERSION: &str = "2.1.0";
const SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
const TOOL_URI: &str = "https://github.com/vertyll/markright";

#[derive(Serialize)]
struct Log<'a> {
	version: &'static str,
	#[serde(rename = "$schema")]
	schema: &'static str,
	runs: [Run<'a>; 1],
}

#[derive(Serialize)]
struct Run<'a> {
	tool: Tool<'a>,
	results: Vec<Finding<'a>>,
}

#[derive(Serialize)]
struct Tool<'a> {
	driver: Driver<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Driver<'a> {
	name: &'static str,
	information_uri: &'static str,
	rules: Vec<RuleEntry<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleEntry<'a> {
	id: &'a str,
	short_description: Message<'a>,
}

#[derive(Serialize)]
struct Message<'a> {
	text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Finding<'a> {
	rule_id: &'a str,
	level: &'static str,
	message: Message<'a>,
	locations: [Location<'a>; 1],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Location<'a> {
	physical_location: PhysicalLocation<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PhysicalLocation<'a> {
	artifact_location: ArtifactLocation<'a>,
	region: Region,
}

#[derive(Serialize)]
struct ArtifactLocation<'a> {
	uri: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Region {
	start_line: usize,
}

/// Writes a SARIF 2.1.0 log with one run: every rule, and one result per issue.
pub(super) fn write(w: &mut dyn Write, issues: &[Issue], rules: &[Rule]) -> io::Result<()> {
	let rules = rules.iter().map(|rule| RuleEntry {
		id: rule.name,
		short_description: Message {
			text: &rule.description,
		},
	});
	let results = issues.iter().map(|issue| Finding {
		rule_id: issue.source,
		level: "error",
		message: Message { text: &issue.message },
		locations: [Location {
			physical_location: PhysicalLocation {
				artifact_location: ArtifactLocation { uri: &issue.path },
				region: Region { start_line: issue.line },
			},
		}],
	});
	let log = Log {
		version: VERSION,
		schema: SCHEMA,
		runs: [Run {
			tool: Tool {
				driver: Driver {
					name: "markright",
					information_uri: TOOL_URI,
					rules: rules.collect(),
				},
			},
			results: results.collect(),
		}],
	};
	serde_json::to_writer_pretty(&mut *w, &log)?;
	w.write_all(b"\n")
}
