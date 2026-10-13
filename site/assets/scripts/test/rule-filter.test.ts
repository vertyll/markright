import assert from "node:assert/strict";
import { test } from "node:test";
import { matchesRule, ruleFromDataset } from "../lib/rule-filter.ts";

const links = ruleFromDataset({ text: "links a relative link points to a file", flags: "" });
const alerts = ruleFromDataset({ text: "alerts an alert tag is an allowed type", flags: "settings" });
const unused = ruleFromDataset({
  text: "unused-definitions every reference definition is used",
  flags: "new settings",
});

test("an empty query and no filters match every rule", () => {
  for (const rule of [links, alerts, unused]) assert.equal(matchesRule(rule, "", []), true);
});

test("the query matches the name or the description, ignoring case and spaces around it", () => {
  assert.equal(matchesRule(links, "  LINK ", []), true);
  assert.equal(matchesRule(links, "alert", []), false);
  assert.equal(matchesRule(alerts, "allowed type", []), true);
});

test("every active filter must apply", () => {
  assert.equal(matchesRule(alerts, "", ["settings"]), true);
  assert.equal(matchesRule(alerts, "", ["settings", "new"]), false);
  assert.equal(matchesRule(unused, "", ["settings", "new"]), true);
  assert.equal(matchesRule(links, "", ["new"]), false);
});

test("a card without data reads as a rule with no text and no flags", () => {
  const rule = ruleFromDataset({});
  assert.equal(rule.text, "");
  assert.equal(rule.flags.size, 0);
  assert.equal(matchesRule(rule, "", []), true);
});
