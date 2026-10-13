//! The Checkstyle report, an XML file that CI servers and code review tools read.

use std::io::{self, Write};

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};

use crate::issue::Issue;

/// Writes a Checkstyle report: one `file` element per document, in the order of the issues, each holding its errors.
pub(super) fn write(w: &mut dyn Write, issues: &[Issue]) -> io::Result<()> {
	let mut xml = Writer::new_with_indent(w, b' ', 2);
	xml.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
	xml.write_event(Event::Start(
		BytesStart::new("checkstyle").with_attributes([("version", "5.0")]),
	))?;
	for file in issues.chunk_by(|a, b| a.path == b.path) {
		xml.write_event(Event::Start(
			BytesStart::new("file").with_attributes([("name", file[0].path.as_str())]),
		))?;
		for issue in file {
			let line = issue.line.to_string();
			let source = format!("markright.{}", issue.source);
			let error = BytesStart::new("error").with_attributes([
				("line", line.as_str()),
				("severity", "error"),
				("message", issue.message.as_str()),
				("source", source.as_str()),
			]);
			xml.write_event(Event::Empty(error))?;
		}
		xml.write_event(Event::End(BytesEnd::new("file")))?;
	}
	xml.write_event(Event::End(BytesEnd::new("checkstyle")))?;
	xml.into_inner().write_all(b"\n")
}
