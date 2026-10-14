//! Mermaid diagrams as pictures: each `mermaid` code block of a document becomes, on its page, the diagram drawn as
//! SVG twice, in a light and a dark theme, of which the stylesheet shows the one that matches the page's. GitHub draws
//! the same blocks from the documents themselves.

use std::fmt::Write;
use std::io;

use markright_core::markdown::Document;
use merman::{OperationControl, RenderOutput, RenderRequest, Renderer, SvgRequest};

/// The Mermaid themes each diagram is drawn in, by the class the stylesheet shows it by.
const THEMES: [(&str, &str); 2] = [("diagram-light", "neutral"), ("diagram-dark", "dark")];

/// Replaces every `mermaid` code block of `document`, whose lines are `lines`, with its pictures. `page` names the page,
/// so each picture's ids, which its styles refer to, are its own on the site.
pub(super) fn draw(document: &Document, lines: &mut [String], page: &str) -> io::Result<()> {
	let renderer = Renderer::new();
	let blocks = document.fences.iter().filter(|fence| fence.info == "mermaid");
	for (number, fence) in blocks.enumerate() {
		// The block runs from its opening fence through the code lines after it, its closing fence the last of them.
		let open = fence.line - 1;
		let close = (open + 1..document.lines.len())
			.take_while(|&i| document.lines[i].code)
			.last()
			.unwrap_or(open);
		let source = lines[open + 1..close].join("\n");
		let mut figure = String::from("<figure class=\"diagram\">");
		for (class, theme) in THEMES {
			let id = format!("{}-diagram-{number}-{theme}", page.replace(['/', '.', '_'], "-"));
			let svg = svg(&renderer, &source, theme, &id)?;
			let _ = write!(figure, "<div class=\"{class}\">{}</div>", svg.replace('\n', " "));
		}
		figure.push_str("</figure>");
		lines[open] = figure;
		for line in &mut lines[open + 1..=close] {
			line.clear();
		}
	}
	Ok(())
}

/// The diagram `source` drawn as SVG in the Mermaid theme `theme`, with `id` for the ids inside it.
fn svg(renderer: &Renderer, source: &str, theme: &str, id: &str) -> io::Result<String> {
	let themed = format!("%%{{init: {{\"theme\": \"{theme}\"}}}}%%\n{source}");
	let mut request = SvgRequest::default();
	request.options.diagram_id = Some(id.to_owned());
	let output = renderer
		.render(RenderRequest::svg(&themed, OperationControl::new(), request))
		.map_err(|e| io::Error::other(format!("drawing a diagram: {e}")))?;
	match output {
		RenderOutput::Svg(Some(svg)) => Ok(svg.svg().to_owned()),
		_ => Err(io::Error::other("a mermaid block holds no diagram")),
	}
}
