use super::{Rule, Streams, write};
use crate::config::{Destination, Format, Formats};
use crate::issue::{Finding, Issue};

fn sample() -> Vec<Issue> {
	vec![
		Finding::new("docs/a.md", 4, "broken link: \"x\".md <&>").found_by("links"),
		Finding::new("docs/a.md", 9, "ASCII arrow").found_by("arrows"),
		Finding::new("README.md", 1, "does not link GLOSSARY.md").found_by("readme-links"),
	]
}

fn rules() -> Vec<Rule> {
	let rules = [
		("links", "a link resolves"),
		("arrows", "unicode arrows"),
		("readme-links", "README links"),
	];
	rules
		.map(|(name, description)| Rule {
			name,
			description: description.to_owned(),
		})
		.to_vec()
}

fn to(path: Destination) -> Format {
	Format { path }
}

/// Formats with only the format `set` sets.
fn only(set: impl FnOnce(&mut Formats)) -> Formats {
	let mut formats = Formats {
		text: None,
		checkstyle: None,
		sarif: None,
	};
	set(&mut formats);
	formats
}

fn report(formats: &Formats, issues: &[Issue]) -> (String, String) {
	let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
	write(
		formats,
		issues,
		&rules(),
		&mut Streams {
			stdout: &mut stdout,
			stderr: &mut stderr,
		},
	)
	.unwrap();
	(String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap())
}

#[test]
fn text() {
	let formats = only(|f| f.text = Some(to(Destination::Stdout)));
	let want = "docs/a.md:4: broken link: \"x\".md <&> (links)\ndocs/a.md:9: ASCII arrow (arrows)\n\
	            README.md:1: does not link GLOSSARY.md (readme-links)\nmarkright: 3 issue(s)\n";
	assert_eq!(report(&formats, &sample()).0, want);
	assert_eq!(report(&formats, &[]).0, "markright: 0 issues\n");
}

#[test]
fn checkstyle() {
	let formats = only(|f| f.checkstyle = Some(to(Destination::Stderr)));
	let (_, xml) = report(&formats, &sample());
	let want = r#"<?xml version="1.0" encoding="UTF-8"?>
<checkstyle version="5.0">
  <file name="docs/a.md">
    <error line="4" severity="error" message="broken link: &quot;x&quot;.md &lt;&amp;&gt;" source="markright.links"/>
    <error line="9" severity="error" message="ASCII arrow" source="markright.arrows"/>
  </file>
  <file name="README.md">
    <error line="1" severity="error" message="does not link GLOSSARY.md" source="markright.readme-links"/>
  </file>
</checkstyle>
"#;
	assert_eq!(xml, want);
}

#[test]
fn sarif() {
	let formats = only(|f| f.sarif = Some(to(Destination::Stdout)));
	let log: serde_json::Value = serde_json::from_str(&report(&formats, &sample()).0).unwrap();
	assert_eq!(log["version"], "2.1.0");
	let run = &log["runs"][0];
	assert_eq!(run["tool"]["driver"]["rules"].as_array().map(Vec::len), Some(3));
	let result = &run["results"][0];
	assert_eq!(result["ruleId"], "links");
	assert_eq!(
		result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
		"docs/a.md"
	);
	assert_eq!(result["locations"][0]["physicalLocation"]["region"]["startLine"], 4);
	let empty: serde_json::Value = serde_json::from_str(&report(&formats, &[]).0).unwrap();
	assert_eq!(
		empty["runs"][0]["results"],
		serde_json::json!([]),
		"an empty run still lists its results"
	);
}

#[test]
fn files_and_several_formats() {
	let dir = tempfile::tempdir().unwrap();
	let formats = Formats {
		text: Some(to(Destination::Stdout)),
		checkstyle: Some(to(Destination::File(dir.path().join("markright.xml")))),
		sarif: Some(to(Destination::File(dir.path().join("markright.sarif")))),
	};
	assert!(report(&formats, &sample()).0.ends_with("markright: 3 issue(s)\n"));
	for name in ["markright.xml", "markright.sarif"] {
		assert!(
			std::fs::read_to_string(dir.path().join(name))
				.unwrap()
				.contains("docs/a.md"),
			"{name}"
		);
	}
	let unwritable = only(|f| f.sarif = Some(to(Destination::File(dir.path().join("missing/x.sarif")))));
	let error = write(
		&unwritable,
		&sample(),
		&rules(),
		&mut Streams {
			stdout: &mut Vec::new(),
			stderr: &mut Vec::new(),
		},
	);
	assert!(
		error
			.unwrap_err()
			.to_string()
			.starts_with("writing the sarif report to ")
	);
}
