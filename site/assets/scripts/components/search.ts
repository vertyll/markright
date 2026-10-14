import { debounce } from "../lib/playground.ts";
import { parseIndex, type SearchEntry, type SearchHit, search } from "../lib/search.ts";

/** A search result: the page's title and the text around the first match, as a link. */
function result(hit: SearchHit, index: number): HTMLLIElement {
  const item = document.createElement("li");
  const link = document.createElement("a");
  link.href = hit.entry.url;
  link.id = `search-result-${index}`;
  const title = document.createElement("span");
  title.className = "search-result-title";
  title.textContent = hit.entry.title;
  const excerpt = document.createElement("span");
  excerpt.className = "search-result-excerpt";
  const mark = document.createElement("mark");
  mark.textContent = hit.excerpt.match;
  excerpt.append(hit.excerpt.before, mark, hit.excerpt.after);
  link.append(title, excerpt);
  item.append(link);
  return item;
}

/** Whether a key press happens while the reader types, so it is theirs and not a shortcut. */
function typing(event: KeyboardEvent): boolean {
  const target = event.target;
  return (
    target instanceof HTMLElement && (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  );
}

/** The search: the header's button, / or Ctrl+K open it, and it loads the index the first time. */
export function initSearch(): void {
  const dialog = document.querySelector<HTMLDialogElement>("[data-search]");
  const input = dialog?.querySelector<HTMLInputElement>("[data-search-input]");
  const results = dialog?.querySelector<HTMLUListElement>("[data-search-results]");
  const status = dialog?.querySelector<HTMLElement>("[data-search-status]");
  const openers = document.querySelectorAll<HTMLButtonElement>("[data-search-open]");
  if (!dialog || !input || !results || !status) return;
  let index: Promise<SearchEntry[]> | undefined;
  let selected = -1;
  const links = () => [...results.querySelectorAll<HTMLAnchorElement>("a")];
  const select = (next: number) => {
    const all = links();
    selected = all.length === 0 ? -1 : (next + all.length) % all.length;
    for (const [i, link] of all.entries()) link.setAttribute("aria-selected", String(i === selected));
    all[selected]?.scrollIntoView({ block: "nearest" });
  };
  const update = async () => {
    const query = input.value;
    try {
      index ??= fetch(dialog.dataset["index"] ?? "").then(async (response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return parseIndex(await response.json());
      });
      const hits = search(await index, query);
      if (input.value !== query) return;
      results.replaceChildren(...hits.map((hit, position) => result(hit, position)));
      select(0);
      const nothing = query.trim() !== "" && hits.length === 0;
      status.textContent = nothing ? `Nothing found for “${query.trim()}”` : "";
    } catch (error) {
      index = undefined;
      status.textContent = `The search could not load: ${error instanceof Error ? error.message : String(error)}`;
    }
  };
  const open = () => {
    if (!dialog.open) dialog.showModal();
    input.select();
    void update();
  };
  for (const opener of openers) opener.addEventListener("click", open);
  dialog.querySelector("[data-search-close]")?.addEventListener("click", () => dialog.close());
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) dialog.close();
  });
  input.addEventListener(
    "input",
    debounce(() => void update(), 100),
  );
  input.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      select(selected + (event.key === "ArrowDown" ? 1 : -1));
    } else if (event.key === "Enter") {
      const link = links()[selected];
      if (link) {
        event.preventDefault();
        link.click();
      }
    }
  });
  document.addEventListener("keydown", (event) => {
    const shortcut = (event.key === "/" && !typing(event)) || (event.key === "k" && (event.ctrlKey || event.metaKey));
    if (shortcut && !dialog.open) {
      event.preventDefault();
      open();
    }
  });
}
