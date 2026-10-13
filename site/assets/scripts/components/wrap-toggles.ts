import { overflowsUnwrapped } from "../lib/overflow.ts";
import type { Settings } from "../lib/settings.ts";
import { icon } from "./icon.ts";

const wrappedClass = "wrapped";

/** The settings that choose whether code blocks and tables wrap. */
type WrapKind = "code" | "table";

interface Toggle {
  host: HTMLElement;
  scroller: HTMLElement;
  button: HTMLButtonElement;
}

/** A button with the icon called name, labelled for assistive technology and in its tooltip. */
function button(className: string, label: string, name: string): HTMLButtonElement {
  const element = document.createElement("button");
  element.type = "button";
  element.className = className;
  element.title = label;
  element.ariaLabel = label;
  element.append(icon(name));
  return element;
}

/** A copy button for a code block: it copies the code and shows a tick for a moment. */
function copyButton(code: HTMLElement): HTMLButtonElement {
  const copy = button("code-button", "Copy the code", "clipboard");
  copy.addEventListener("click", () => {
    navigator.clipboard
      .writeText(code.textContent ?? "")
      .then(() => {
        copy.dataset["copied"] = "";
        copy.replaceChildren(icon("check"));
        setTimeout(() => {
          delete copy.dataset["copied"];
          copy.replaceChildren(icon("clipboard"));
        }, 1500);
      })
      .catch(() => undefined);
  });
  return copy;
}

/** Every code block of the text in a host with its buttons at the top right: wrap, then copy. */
function codeBlocks(): Toggle[] {
  return [...document.querySelectorAll<HTMLPreElement>(".content pre")].map((scroller) => {
    const host = document.createElement("div");
    host.className = "code-block";
    scroller.before(host);
    host.append(scroller);
    const bar = document.createElement("div");
    bar.className = "code-buttons";
    const wrap = button("code-button", "Wrap long lines", "bars-3-bottom-left");
    bar.append(wrap, copyButton(scroller));
    host.append(bar);
    return { host, scroller, button: wrap };
  });
}

/** A button above every table, which is moved into a host that carries the wrapped class. */
function tables(): Toggle[] {
  return [...document.querySelectorAll<HTMLTableElement>(".content table")].map((scroller) => {
    const host = document.createElement("div");
    host.className = "table-host";
    scroller.before(host);
    host.append(scroller);
    const bar = document.createElement("div");
    bar.className = "table-buttons";
    const wrap = button("code-button", "Wrap table cells", "bars-3-bottom-left");
    bar.append(wrap);
    host.prepend(bar);
    return { host, scroller, button: wrap };
  });
}

/** Wraps or unwraps every host of toggles, and shows each button only where its content is too wide. */
function apply(toggles: readonly Toggle[], wrapped: boolean): void {
  for (const { host, scroller, button } of toggles) {
    button.hidden = !overflowsUnwrapped(host, scroller, wrappedClass);
    host.classList.toggle(wrappedClass, wrapped);
    button.setAttribute("aria-pressed", String(wrapped));
  }
}

/** Adds the buttons to code blocks and tables, and keeps every one of a kind in step with its setting. */
export function initWrapToggles(settings: Settings): void {
  const groups: Record<WrapKind, Toggle[]> = { code: codeBlocks(), table: tables() };
  const wrapped = (kind: WrapKind) => settings.get(kind) === "wrap";
  for (const kind of ["code", "table"] as const) {
    const toggles = groups[kind];
    const refresh = () => apply(toggles, wrapped(kind));
    for (const { scroller, button } of toggles) {
      button.addEventListener("click", () => settings.set(kind, wrapped(kind) ? "scroll" : "wrap"));
      new ResizeObserver(refresh).observe(scroller);
    }
    refresh();
  }
  settings.subscribe((name) => {
    if (name === "code" || name === "table") apply(groups[name], wrapped(name));
  });
}
