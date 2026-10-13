import type { Settings } from "../lib/settings.ts";

const inputSelector = "input[data-setting]";

/** Checks, in every settings form, the choice each setting holds. */
function syncForms(settings: Settings): void {
  for (const input of document.querySelectorAll<HTMLInputElement>(inputSelector)) {
    input.checked = settings.get(input.dataset["setting"] ?? "") === input.value;
  }
}

/** Turns the header's settings link into a button that opens the panel under it. */
function initPanel(): void {
  const panel = document.querySelector<HTMLDialogElement>("[data-settings-panel]");
  const trigger = document.querySelector<HTMLAnchorElement>("[data-settings-open]");
  if (!panel || !trigger) return;
  // Closing from the panel, with its button or Escape, gives focus back to the icon that opened it; a click elsewhere
  // leaves focus where the reader put it.
  const close = (returnFocus: boolean) => {
    panel.close();
    trigger.setAttribute("aria-expanded", "false");
    if (returnFocus) trigger.focus();
  };
  panel.querySelector("[data-settings-close]")?.addEventListener("click", () => close(true));
  trigger.setAttribute("aria-haspopup", "dialog");
  trigger.setAttribute("aria-expanded", "false");
  trigger.addEventListener("click", (event) => {
    event.preventDefault();
    if (panel.open) {
      close(false);
      return;
    }
    panel.show();
    trigger.setAttribute("aria-expanded", "true");
    panel.querySelector<HTMLInputElement>("input:checked")?.focus();
  });
  document.addEventListener("click", (event) => {
    const target = event.target;
    if (!panel.open || !(target instanceof Node)) return;
    if (!panel.contains(target) && !trigger.contains(target)) close(false);
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && panel.open) close(true);
  });
}

/** The theme button: it moves to the next theme, from the system's to light, dark and back. */
function initThemeToggle(settings: Settings): void {
  const toggle = document.querySelector<HTMLButtonElement>("[data-theme-toggle]");
  const label = toggle?.querySelector<HTMLElement>("[data-theme-label]");
  const options = settings.definitions.find((definition) => definition.name === "theme")?.options ?? [];
  if (!toggle || !label || options.length === 0) return;
  const sync = () => {
    const current = options.find((option) => option.value === settings.get("theme"));
    label.textContent = `Theme: ${current?.label ?? ""}`;
    toggle.title = label.textContent;
  };
  toggle.addEventListener("click", () => {
    const index = options.findIndex((option) => option.value === settings.get("theme"));
    const next = options[(index + 1) % options.length];
    if (next) settings.set("theme", next.value);
  });
  settings.subscribe((name) => {
    if (name === "theme") sync();
  });
  sync();
}

/** Connects every settings form, in the panel and on the settings page, and the theme button to the settings. */
export function initSettings(settings: Settings): void {
  initPanel();
  initThemeToggle(settings);
  document.addEventListener("change", (event) => {
    const input = event.target instanceof Element ? event.target.closest<HTMLInputElement>(inputSelector) : null;
    if (input?.dataset["setting"]) settings.set(input.dataset["setting"], input.value);
  });
  settings.subscribe(() => syncForms(settings));
  syncForms(settings);
}
