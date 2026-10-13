import { isRecord } from "../lib/guards.ts";
import {
  configFile,
  debounce,
  errorMessage,
  isSyntax,
  lineRange,
  type Outcome,
  parseOutcome,
  type Syntax,
  summary,
  tabAfterKey,
} from "../lib/playground.ts";
import type { Settings } from "../lib/settings.ts";

type Check = (markdown: string, configuration: string, syntax: Syntax) => Outcome;

/** What the module wasm-bindgen writes for markright exports: its start and its check. */
interface Module {
  default(options: { module_or_path: BufferSource }): Promise<unknown>;
  check(markdown: string, configuration: string, syntax: string): unknown;
}

/** Whether value is the module markright's WebAssembly comes with. */
function isModule(value: unknown): value is Module {
  return isRecord(value) && typeof value["default"] === "function" && typeof value["check"] === "function";
}

/** Loads markright, checking the WebAssembly against the hash the page gives, and returns its check. */
async function loadMarkright(root: HTMLElement): Promise<Check> {
  const { module = "", wasm = "", wasmIntegrity = "" } = root.dataset;
  // The module is a file wasm-bindgen writes when the site is built, so it is typed by Module and checked on arrival.
  const loading: Promise<unknown> = import(module);
  const [imported, response] = await Promise.all([loading, fetch(wasm, { integrity: wasmIntegrity })]);
  if (!response.ok) throw new Error(`could not load markright: HTTP ${response.status}`);
  if (!isModule(imported)) throw new Error("markright did not load");
  await imported.default({ module_or_path: await response.arrayBuffer() });
  return (markdown, configuration, syntax) => parseOutcome(imported.check(markdown, configuration, syntax));
}

/** An issue in the list of results: a button that selects its line in the editor. */
function issueItem(issue: Outcome["issues"][number], editor: HTMLTextAreaElement): HTMLLIElement {
  const item = document.createElement("li");
  const button = document.createElement("button");
  button.type = "button";
  button.title = `Go to line ${issue.line}`;
  for (const [className, text] of [
    ["playground-issue-line", `${issue.line}`],
    ["playground-issue-message", issue.message],
    ["playground-issue-source", issue.source],
  ] as const) {
    const part = document.createElement("span");
    part.className = className;
    part.textContent = text;
    button.append(part);
  }
  button.addEventListener("click", () => {
    const [start, end] = lineRange(editor.value, issue.line);
    editor.focus();
    editor.setSelectionRange(start, end);
  });
  item.append(button);
  return item;
}

/** Makes the editor as tall as its text, up to the maximum height its stylesheet sets; past that it scrolls. */
function fitHeight(editor: HTMLTextAreaElement): void {
  editor.style.height = "auto";
  editor.style.height = `${editor.scrollHeight + editor.offsetHeight - editor.clientHeight}px`;
}

/** The tabs over the results, markright check and markright fmt, which show one panel at a time. */
function initTabs(root: HTMLElement): void {
  const tabs = [...root.querySelectorAll<HTMLButtonElement>("[data-playground-tab]")];
  const select = (chosen: HTMLButtonElement) => {
    for (const tab of tabs) {
      const selected = tab === chosen;
      tab.ariaSelected = String(selected);
      tab.tabIndex = selected ? 0 : -1;
      const panel = document.getElementById(tab.getAttribute("aria-controls") ?? "");
      if (panel) panel.hidden = !selected;
    }
  };
  tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => select(tab));
    tab.addEventListener("keydown", (event) => {
      const next = tabs[tabAfterKey(event.key, index, tabs.length) ?? -1];
      if (next) {
        event.preventDefault();
        select(next);
        next.focus();
      }
    });
  });
}

/**
 * The wrap buttons, by the editor and by the results: they follow the Code blocks setting, which a stylesheet applies
 * to the editors, the issues and the formatted document, and change it like the button of any code block.
 */
function initWrap(buttons: readonly HTMLButtonElement[], settings: Settings, refit: () => void): void {
  const sync = () => {
    const wrapped = String(settings.get("code") === "wrap");
    for (const button of buttons) button.ariaPressed = wrapped;
    refit();
  };
  for (const button of buttons) {
    button.addEventListener("click", () => settings.set("code", settings.get("code") === "wrap" ? "scroll" : "wrap"));
  }
  settings.subscribe((name) => {
    if (name === "code") sync();
  });
  sync();
}

/** The syntax the configuration is written in, as the reader chose it, and the file name and example that follow it. */
function initSyntax(root: HTMLElement, config: HTMLTextAreaElement, changed: () => void): () => Syntax {
  const choices = [...root.querySelectorAll<HTMLInputElement>("[data-playground-syntax]")];
  const name = root.querySelector<HTMLElement>("[data-playground-config-name]");
  const current = (): Syntax => {
    const value = choices.find((choice) => choice.checked)?.value ?? "yaml";
    return isSyntax(value) ? value : "yaml";
  };
  const sync = () => {
    const syntax = current();
    if (name) name.textContent = configFile(syntax);
    config.placeholder = config.dataset[syntax === "yaml" ? "placeholderYaml" : "placeholderToml"] ?? "";
  };
  for (const choice of choices) {
    choice.addEventListener("change", () => {
      sync();
      changed();
    });
  }
  sync();
  return current;
}

/** Starts the playground on the page, if there is one: it checks the document on every change. */
export function initPlayground(settings: Settings): void {
  const root = document.querySelector<HTMLElement>("[data-playground]");
  const editor = root?.querySelector<HTMLTextAreaElement>("[data-playground-document]");
  const config = root?.querySelector<HTMLTextAreaElement>("[data-playground-config]");
  const issues = root?.querySelector<HTMLOListElement>("[data-playground-issues]");
  const status = root?.querySelector<HTMLElement>("[data-playground-status]");
  const formatted = root?.querySelector<HTMLPreElement>("[data-playground-formatted]");
  const apply = root?.querySelector<HTMLButtonElement>("[data-playground-apply]");
  const wraps = [...(root?.querySelectorAll<HTMLButtonElement>("[data-playground-wrap]") ?? [])];
  if (!root || !editor || !config || !issues || !status || !formatted || !apply) return;
  initTabs(root);
  for (const textarea of [editor, config]) {
    const fit = () => fitHeight(textarea);
    textarea.addEventListener("input", fit);
    new ResizeObserver(fit).observe(textarea.parentElement ?? textarea);
    fit();
  }
  initWrap(wraps, settings, () => [editor, config].forEach(fitHeight));
  config.closest("details")?.addEventListener("toggle", () => fitHeight(config));
  let check: Check | undefined;
  const update = () => {
    if (!check) return;
    try {
      const outcome = check(editor.value, config.value, syntax());
      issues.replaceChildren(...outcome.issues.map((issue) => issueItem(issue, editor)));
      formatted.textContent = outcome.formatted;
      apply.disabled = outcome.formatted === editor.value;
      status.dataset["state"] = "ok";
      status.textContent = summary(outcome);
    } catch (error) {
      // markright throws why it could not check the document, such as a configuration that is not valid.
      issues.replaceChildren();
      formatted.textContent = "";
      apply.disabled = true;
      status.dataset["state"] = "error";
      status.textContent = errorMessage(error);
    }
  };
  const syntax = initSyntax(root, config, update);
  const later = debounce(update, 150);
  editor.addEventListener("input", later);
  config.addEventListener("input", later);
  apply.addEventListener("click", () => {
    editor.value = formatted.textContent ?? editor.value;
    fitHeight(editor);
    update();
  });
  loadMarkright(root)
    .then((loaded) => {
      check = loaded;
      update();
    })
    .catch((error: unknown) => {
      status.dataset["state"] = "error";
      status.textContent = errorMessage(error);
    });
}
