const svg = "http://www.w3.org/2000/svg";

/** A decorative icon from the site's icons, as the templates draw them: the body says where the icons are. */
export function icon(name: string, size = 16): SVGSVGElement {
  const element = document.createElementNS(svg, "svg");
  element.setAttribute("class", "icon");
  element.setAttribute("width", String(size));
  element.setAttribute("height", String(size));
  element.setAttribute("aria-hidden", "true");
  element.setAttribute("focusable", "false");
  const use = document.createElementNS(svg, "use");
  use.setAttribute("href", `${document.body.dataset["icons"] ?? ""}#${name}`);
  element.append(use);
  return element;
}
