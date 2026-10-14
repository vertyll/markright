import assert from "node:assert/strict";
import { test } from "node:test";
import { excerpt, parseIndex, type SearchEntry, score, search, terms } from "../lib/search.ts";

const page = (title: string, body: string, description = ""): SearchEntry => ({
  url: `https://example.com/${title}/`,
  title,
  description,
  body,
});

const entries = [
  page("links", "A relative link must lead to a file. Broken links are reported.", "every link resolves"),
  page("Configuration", "The links linter has no settings.\nThe line-length linter has a max."),
  page("arrows", "ASCII arrows become Unicode arrows."),
];

test("parseIndex reads the index and rejects anything else", () => {
  assert.deepEqual(parseIndex(entries), entries);
  for (const value of [null, {}, [{ url: "x" }], [{ url: 1, title: "", description: "", body: "" }]]) {
    assert.throws(() => parseIndex(value), TypeError, JSON.stringify(value));
  }
});

test("terms are the words of a query, in lower case", () => {
  assert.deepEqual(terms("  Broken   Links "), ["broken", "links"]);
  assert.deepEqual(terms(" "), []);
});

test("a page matches when it holds every word, best first", () => {
  assert.deepEqual(
    search(entries, "links").map((hit) => hit.entry.title),
    ["links", "Configuration"],
  );
  assert.deepEqual(
    search(entries, "linter max").map((hit) => hit.entry.title),
    ["Configuration"],
  );
  assert.deepEqual(search(entries, "links unicode"), []);
  assert.deepEqual(search(entries, "   "), []);
});

test("the title counts more than the text", () => {
  const [first, second] = search([page("other", "arrows arrows arrows"), page("arrows", "")], "arrows");
  assert.equal(first?.entry.title, "arrows");
  assert.equal(second?.entry.title, "other");
});

test("search keeps at most limit pages", () => {
  const many = Array.from({ length: 20 }, (_, i) => page(`page ${i}`, "markright"));
  assert.equal(search(many, "markright").length, 10);
  assert.equal(search(many, "markright", 3).length, 3);
});

test("an excerpt is the text around the first match, on one line", () => {
  assert.deepEqual(excerpt("One\ntwo  three", "two"), { before: "One ", match: "two", after: " three" });
  const long = `${"a".repeat(100)} needle ${"b".repeat(100)}`;
  const cut = excerpt(long, "needle");
  assert.equal(cut.match, "needle");
  assert.ok(cut.before.startsWith("…") && cut.after.endsWith("…"));
  assert.equal(cut.before.length, 61);
  assert.deepEqual(excerpt("Nothing here", "absent"), { before: "", match: "", after: "Nothing here" });
  assert.deepEqual(excerpt("Kept as It Was", "it"), { before: "Kept as ", match: "It", after: " Was" });
});

test("score weighs the title, the description and the text, and is 0 when a word is missing", () => {
  const entry = page("arrows", "arrows arrows", "turns arrows into characters");
  assert.equal(score(entry, ["arrows"]), 20 + 10 + 4 + 2);
  assert.equal(score(entry, ["characters"]), 4);
  assert.equal(score(entry, ["arrows", "missing"]), 0);
});
