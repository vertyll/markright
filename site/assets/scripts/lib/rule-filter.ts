/** What a rule list filters on: the rule's name and description in lower case, and its flags. */
export interface Rule {
  readonly text: string;
  readonly flags: ReadonlySet<string>;
}

/** Whether the rule contains the query and has every active filter. */
export function matchesRule(rule: Rule, query: string, filters: readonly string[]): boolean {
  const text = query.trim().toLowerCase();
  return rule.text.includes(text) && filters.every((filter) => rule.flags.has(filter));
}

/** Reads a rule from a card's data attributes: data-text, and data-flags separated by spaces. */
export function ruleFromDataset(dataset: Readonly<Record<string, string | undefined>>): Rule {
  return {
    text: dataset["text"] ?? "",
    flags: new Set((dataset["flags"] ?? "").split(" ").filter(Boolean)),
  };
}
