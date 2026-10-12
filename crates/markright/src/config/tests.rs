use super::{Config, ConfigError, Destination, ExclusionKey, Section, Syntax, TOML_FILE, YAML_FILE as FILE};
use crate::markdown::Marker;
use crate::{formatters, linters};

fn parse(text: &str) -> Result<Config, ConfigError> {
	Config::parse(text, FILE, Syntax::Yaml)
}

fn load(text: &str) -> Config {
	parse(text).unwrap_or_else(|e| panic!("{e}"))
}

fn max(config: &Config) -> usize {
	config.linters.settings.line_length.max.get()
}

#[test]
fn a_missing_file_gives_the_defaults() {
	let dir = tempfile::tempdir().unwrap();
	let config = Config::load(&dir.path().join(FILE)).unwrap();
	assert_eq!(
		(max(&config), config.linters.settings.readme_links.links.len()),
		(120, 3)
	);
	assert!(linters::ALL.iter().all(|l| config.linter_enabled(l.about().name)));
	assert!(formatters::ALL.iter().all(|f| config.formatter_enabled(f.about().name)));
	assert_eq!(config.output.formats.text.map(|t| t.path), Some(Destination::Stdout));
}

#[test]
fn a_file_is_read_from_disk() {
	let dir = tempfile::tempdir().unwrap();
	let path = dir.path().join(FILE);
	std::fs::write(&path, "version: \"1\"\nlinters:\n  disable: [links]\n").unwrap();
	assert!(!Config::load(&path).unwrap().linter_enabled("links"));
	std::fs::write(&path, "version: \"2\"\n").unwrap();
	let error = Config::load(&path).unwrap_err().to_string();
	assert_eq!(error, ".markright.yaml: version must be \"1\", got \"2\"");
}

#[test]
fn the_file_overrides_only_the_keys_it_sets() {
	let config = load(
		"version: \"1\"\nlinters:\n  disable: [line-length]\n  settings:\n    readme-links:\n      links: [c]\n\
		 formatters:\n  disable: [tables]\n",
	);
	assert!(!config.linter_enabled("line-length") && config.linter_enabled("links"));
	assert!(!config.formatter_enabled("tables") && config.formatter_enabled("arrows"));
	let readme = &config.linters.settings.readme_links;
	assert_eq!(
		(max(&config), readme.file.as_str(), readme.links.clone()),
		(120, "README.md", vec!["c".to_owned()])
	);
}

#[test]
fn default_none_with_enable() {
	let config = load("version: \"1\"\nlinters:\n  default: none\n  enable: [links]\nformatters:\n  default: none\n");
	assert!(config.linter_enabled("links") && !config.linter_enabled("anchors") && !config.formatter_enabled("arrows"));
}

#[test]
fn empty_sections_are_the_defaults() {
	let config = load("version: \"1\"\nlinters:\nformatters:\n  settings:\n");
	assert!(config.linter_enabled("links") && config.formatter_enabled("arrows"));
}

#[test]
fn exclusions() {
	let config = load(
		"version: \"1\"\nlinters:\n  exclusions:\n    paths: ['^generated/']\n    rules:\n      - path: '^docs/legacy\\.md$'\n        \
		 linters: [line-length]\n      - text: 'broken link: http'\n        linters: [links]\nformatters:\n  exclusions:\n    \
		 paths: ['^CHANGELOG\\.md$']\n",
	);
	let cases = [
		("generated/a.md", "links", "x", true),
		("docs/legacy.md", "line-length", "x", true),
		("docs/legacy.md", "links", "x", false),
		("docs/a.md", "links", "broken link: http.md", true),
		("docs/a.md", "line-length", "x", false),
	];
	for (path, linter, message, want) in cases {
		assert_eq!(
			config.linter_excluded(path, linter, message).0,
			want,
			"{path} {linter} {message}"
		);
	}
	assert!(config.formatter_excluded("CHANGELOG.md").0 && !config.formatter_excluded("README.md").0);
}

#[test]
fn rejects() {
	let cases = [
		("missing version", "linters:\n  disable: [links]\n"),
		("wrong version", "version: \"2\"\n"),
		("unknown key", "version: \"1\"\nlinter:\n  disable: [links]\n"),
		("unknown linter", "version: \"1\"\nlinters:\n  disable: [link]\n"),
		("formatter as linter", "version: \"1\"\nlinters:\n  enable: [tables]\n"),
		(
			"enabled and disabled",
			"version: \"1\"\nformatters:\n  enable: [tables]\n  disable: [tables]\n",
		),
		("bad default", "version: \"1\"\nlinters:\n  default: some\n"),
		(
			"zero length",
			"version: \"1\"\nlinters:\n  settings:\n    line-length:\n      max: 0\n",
		),
		(
			"unknown setting",
			"version: \"1\"\nlinters:\n  settings:\n    line-length:\n      width: 80\n",
		),
		(
			"settings of no linter",
			"version: \"1\"\nlinters:\n  settings:\n    nope:\n      max: 3\n",
		),
		(
			"linter without settings",
			"version: \"1\"\nlinters:\n  settings:\n    links:\n      max: 3\n",
		),
		(
			"setting of a wrong type",
			"version: \"1\"\nlinters:\n  settings:\n    line-length:\n      max: many\n",
		),
		(
			"settings that are no map",
			"version: \"1\"\nlinters:\n  settings:\n    line-length: 80\n",
		),
		(
			"bad formatter setting",
			"version: \"1\"\nformatters:\n  settings:\n    list-marker:\n      marker: \"x\"\n",
		),
		(
			"unknown formatter key",
			"version: \"1\"\nformatters:\n  settings:\n    list-marker:\n      style: x\n",
		),
		(
			"formatter without settings",
			"version: \"1\"\nformatters:\n  settings:\n    tables:\n      mark: x\n",
		),
		(
			"settings of no formatter",
			"version: \"1\"\nformatters:\n  settings:\n    nope:\n      mark: x\n",
		),
		(
			"bad regexp",
			"version: \"1\"\nlinters:\n  exclusions:\n    paths: ['(']\n",
		),
		(
			"empty path",
			"version: \"1\"\nformatters:\n  exclusions:\n    paths-except: ['']\n",
		),
		(
			"one-field rule",
			"version: \"1\"\nlinters:\n  exclusions:\n    rules:\n      - linters: [links]\n",
		),
		(
			"path and path-except",
			"version: \"1\"\nlinters:\n  exclusions:\n    rules:\n      - {path: a, path-except: b, linters: [links]}\n",
		),
		(
			"unknown linter in rule",
			"version: \"1\"\nlinters:\n  exclusions:\n    rules:\n      - {path: x, linters: [nope]}\n",
		),
		(
			"empty output path",
			"version: \"1\"\noutput:\n  formats:\n    sarif:\n      path: \"\"\n",
		),
		(
			"unknown format",
			"version: \"1\"\noutput:\n  formats:\n    json:\n      path: x\n",
		),
		("not YAML", "version: \"1\"\nlinters: [\n"),
		("repeated key", "version: \"1\"\nversion: \"1\"\n"),
	];
	for (name, text) in cases {
		let error = parse(text).map(|_| ()).expect_err(name).to_string();
		assert!(error.starts_with(".markright.yaml: "), "{name}: {error}");
		assert!(!error.contains('\n'), "{name}: {error}");
	}
}

#[test]
fn errors_name_the_line() {
	let cases = [
		(
			"version: \"1\"\nlinters:\n  settings:\n    line-length:\n      width: 80\n",
			5,
			"unknown field `width`, expected one of max",
		),
		(
			"version: \"1\"\nlinters:\n  settings:\n    line-length:\n      max: many\n",
			5,
			"invalid u64",
		),
		(
			"version: \"1\"\nlinters:\n  defaultx: all\n",
			3,
			"unknown field `defaultx`, expected one of default, enable, disable, settings, exclusions",
		),
		(
			"version: \"1\"\nlinters:\n  exclusions:\n    paths: ['(']\n",
			4,
			"invalid regular expression \"(\": unclosed group",
		),
		(
			"version: \"1\"\nlinters:\n  exclusions:\n    rules:\n      - {path: '', linters: [links]}\n",
			5,
			"empty regular expression; leave the key out instead",
		),
	];
	for (text, line, part) in cases {
		match parse(text) {
			Err(ConfigError::Parse { line: got, message, .. }) => {
				assert_eq!(got, line, "{message}");
				assert_eq!(message, part);
			}
			other => panic!("{text:?} gave {other:?}"),
		}
	}
}

#[test]
fn paths_except() {
	let config = load(
		"version: \"1\"\nlinters:\n  exclusions:\n    paths-except: ['^docs/']\n    rules:\n      - path-except: '^docs/api/'\n        \
		 linters: [line-length]\nformatters:\n  exclusions:\n    paths: ['^docs/generated/']\n    paths-except: ['^docs/']\n",
	);
	let cases = [
		("README.md", "links", true),
		("docs/a.md", "links", false),
		("docs/a.md", "line-length", true),
		("docs/api/a.md", "line-length", false),
	];
	for (path, linter, want) in cases {
		assert_eq!(config.linter_excluded(path, linter, "x").0, want, "{path} {linter}");
	}
	for (path, want) in [("README.md", true), ("docs/a.md", false), ("docs/generated/a.md", true)] {
		assert_eq!(config.formatter_excluded(path).0, want, "{path}");
	}
}

#[test]
fn hits_and_watched() {
	let config = load(
		"version: \"1\"\nlinters:\n  exclusions:\n    warn-unused: true\n    paths: ['^used/', '^unused/']\n    rules:\n      \
		 - {path: '^a\\.md$', linters: [links]}\n      - {path: '^b\\.md$', linters: [links]}\nformatters:\n  exclusions:\n    \
		 paths: ['^never/']\n",
	);
	assert_eq!(
		config.linter_excluded("used/x.md", "anchors", "m").1,
		[ExclusionKey::Path {
			section: Section::Linters,
			index: 0
		}]
	);
	assert_eq!(config.linter_excluded("a.md", "links", "m").1, [ExclusionKey::Rule(0)]);
	let watched = config.watched();
	let keys: Vec<String> = watched.iter().map(|w| w.key.to_string()).collect();
	assert_eq!(
		keys,
		[
			"linters.exclusions.paths[0]",
			"linters.exclusions.paths[1]",
			"linters.exclusions.rules[0]",
			"linters.exclusions.rules[1]"
		],
		"the formatters do not ask for warn-unused"
	);
	assert!(watched.iter().all(|w| w.warning.contains(&w.key.to_string())));
	assert_eq!(
		watched[1].warning,
		"linters.exclusions.paths[1]: \"^unused/\" matches no document"
	);
}

#[test]
fn output() {
	let config = load("version: \"1\"\n");
	let f = &config.output.formats;
	assert!(f.text.as_ref().is_some_and(|t| t.path == Destination::Stdout) && f.sarif.is_none());
	let config = load("version: \"1\"\noutput:\n  formats:\n    sarif:\n      path: markright.sarif\n");
	let f = &config.output.formats;
	assert!(
		f.text.is_none()
			&& f.sarif
				.as_ref()
				.is_some_and(|s| s.path == Destination::File("markright.sarif".into()))
	);
}

#[test]
fn settings_are_read_into_their_own_value() {
	let first = load("version: \"1\"\nlinters:\n  settings:\n    line-length:\n      max: 80\n");
	let second = load("version: \"1\"\n");
	assert_eq!((max(&first), max(&second)), (80, 120));
}

#[test]
fn formatter_settings_are_read_like_linter_settings() {
	let set = load("version: \"1\"\nformatters:\n  settings:\n    list-marker:\n      marker: \"*\"\n");
	let unset = load("version: \"1\"\n");
	assert_eq!(set.formatters.settings.list_marker.marker, Marker::Star);
	assert_eq!(unset.formatters.settings.list_marker.marker, Marker::Dash);
}

#[test]
fn a_number_is_a_version() {
	assert!(parse("version: 1\n").is_ok());
}

fn parse_toml(text: &str) -> Result<Config, ConfigError> {
	Config::parse(text, TOML_FILE, Syntax::Toml)
}

#[test]
fn toml_says_what_yaml_says() {
	let yaml = load(
		"version: \"1\"\noutput:\n  formats:\n    sarif:\n      path: out.sarif\nlinters:\n  disable: [links]\n  settings:\n    \
		 line-length:\n      max: 80\n  exclusions:\n    paths: ['^gen/']\n    rules:\n      - {path: '^a\\.md$', linters: [anchors]}\n\
		 formatters:\n  default: none\n  enable: [arrows]\n  settings:\n    list-marker:\n      marker: \"*\"\n",
	);
	let toml = parse_toml(
		"version = \"1\"\n\n[output.formats.sarif]\npath = \"out.sarif\"\n\n[linters]\ndisable = [\"links\"]\n\n\
		 [linters.settings.line-length]\nmax = 80\n\n[linters.exclusions]\npaths = ['^gen/']\n\n[[linters.exclusions.rules]]\n\
		 path = '^a\\.md$'\nlinters = [\"anchors\"]\n\n[formatters]\ndefault = \"none\"\nenable = [\"arrows\"]\n\n\
		 [formatters.settings.list-marker]\nmarker = \"*\"\n",
	)
	.unwrap_or_else(|e| panic!("{e}"));
	let json = |c: &Config| serde_json::to_value(c).unwrap();
	assert_eq!(json(&toml), json(&yaml));
	assert_eq!(
		(max(&toml), toml.formatters.settings.list_marker.marker),
		(80, Marker::Star)
	);
	assert!(toml.linter_excluded("a.md", "anchors", "x").0 && !toml.linter_enabled("links"));
}

#[test]
fn toml_errors_name_the_line() {
	let cases = [
		(
			"version = \"1\"\n\n[linters]\ndefaultx = \"all\"\n",
			4,
			"unknown field `defaultx`, expected one of `default`, `enable`, `disable`, `settings`, `exclusions`",
		),
		(
			"version = \"1\"\n\n[linters.settings.line-length]\nwidth = 80\n",
			4,
			"unknown field `width`, expected `max`",
		),
		(
			"version = \"1\"\n\n[linters.settings.nope]\n",
			3,
			"unknown field `nope`, expected one of `line-length`, `index`, `readme-links`, `alerts`",
		),
		("version = \"1\"\nlinters = [\n", 2, "unclosed array, expected `]`"),
		("version = 1\n", 1, "invalid type: integer `1`, expected a string"),
	];
	for (text, line, part) in cases {
		match parse_toml(text) {
			Err(ConfigError::Parse {
				line: got,
				message,
				file,
			}) => {
				assert_eq!((got, file.as_str()), (line, TOML_FILE), "{message}");
				assert_eq!(message, part);
			}
			other => panic!("{text:?} gave {other:?}"),
		}
	}
	let error = parse_toml("").map(|_| ()).unwrap_err().to_string();
	assert_eq!(error, ".markright.toml: line 1: missing field `version`");
	let error = parse("# nothing\n").map(|_| ()).unwrap_err().to_string();
	assert_eq!(
		error, ".markright.yaml: line 1: missing field `version`",
		"an empty YAML file says the same"
	);
}

#[test]
fn the_root_holds_one_file_at_most() {
	let dir = tempfile::tempdir().unwrap();
	assert_eq!(Config::locate(dir.path()).unwrap(), None);
	std::fs::write(
		dir.path().join(TOML_FILE),
		"version = \"1\"\n[linters]\ndisable = [\"links\"]\n",
	)
	.unwrap();
	let found = Config::locate(dir.path()).unwrap().expect("the TOML file");
	assert_eq!(found, dir.path().join(TOML_FILE));
	assert!(
		!Config::load(&found).unwrap().linter_enabled("links"),
		"read as TOML by its extension"
	);
	std::fs::write(dir.path().join(FILE), "version: \"1\"\n").unwrap();
	assert!(matches!(Config::locate(dir.path()), Err(ConfigError::Ambiguous { root }) if root == dir.path()));
}
