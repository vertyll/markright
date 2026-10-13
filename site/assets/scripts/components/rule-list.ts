import { matchesRule, ruleFromDataset } from "../lib/rule-filter.ts";

/** Filters the cards of list by its search box and chips, and counts those shown. */
function initList(list: HTMLElement): void {
  const search = list.querySelector<HTMLInputElement>("[data-rule-search]");
  const count = list.querySelector<HTMLElement>("[data-rule-count]");
  const reset = list.querySelector<HTMLButtonElement>("[data-rule-reset]");
  if (!search || !count || !reset) return;
  const chips = [...list.querySelectorAll<HTMLButtonElement>("[data-rule-filter]")];
  const cards = [...list.querySelectorAll<HTMLElement>("[data-rule]")].map((card) => ({
    card,
    rule: ruleFromDataset(card.dataset),
  }));
  const update = () => {
    const active = chips.filter((chip) => chip.ariaPressed === "true").map((chip) => chip.dataset["ruleFilter"] ?? "");
    let shown = 0;
    for (const { card, rule } of cards) {
      card.hidden = !matchesRule(rule, search.value, active);
      if (!card.hidden) shown++;
    }
    count.textContent = `${shown} of ${cards.length}`;
  };
  for (const chip of chips) {
    chip.addEventListener("click", () => {
      chip.ariaPressed = String(chip.ariaPressed !== "true");
      update();
    });
  }
  reset.addEventListener("click", () => {
    search.value = "";
    for (const chip of chips) chip.ariaPressed = "false";
    update();
  });
  search.addEventListener("input", update);
  update();
}

/** Makes every rule list on the page searchable and filterable. */
export function initRuleLists(): void {
  document.querySelectorAll<HTMLElement>("[data-rule-list]").forEach(initList);
}
