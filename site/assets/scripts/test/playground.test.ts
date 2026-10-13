import assert from "node:assert/strict";
import { mock, test } from "node:test";
import {
  configFile,
  debounce,
  errorMessage,
  isSyntax,
  lineRange,
  parseOutcome,
  summary,
  tabAfterKey,
} from "../lib/playground.ts";

const outcome = {
  issues: [{ line: 3, source: "arrows", message: "ASCII arrow or missing space around an arrow" }],
  report: "doc.md:3: ASCII arrow or missing space around an arrow (arrows)\nmarkright: 1 issue(s)\n",
  formatted: "# Title\n",
};

test("parseOutcome reads what the module returns", () => {
  assert.deepEqual(parseOutcome(structuredClone(outcome)), outcome);
  assert.deepEqual(parseOutcome({ ...outcome, extra: true }), outcome);
});

test("parseOutcome rejects anything else", () => {
  for (const value of [null, 1, "x", { issues: [] }, { issues: [{ line: "3" }], report: "", formatted: "" }]) {
    assert.throws(() => parseOutcome(value), TypeError, JSON.stringify(value));
  }
});

test("errorMessage reads what was thrown", () => {
  assert.equal(errorMessage(new Error("configuration: .markright.yaml: bad")), "configuration: .markright.yaml: bad");
  assert.equal(errorMessage("plain"), "plain");
});

test("the syntaxes and their files", () => {
  assert.equal(configFile("yaml"), ".markright.yaml");
  assert.equal(configFile("toml"), ".markright.toml");
  assert.ok(isSyntax("yaml") && isSyntax("toml"));
  assert.ok(!isSyntax("json") && !isSyntax(""));
});

test("summary is the report's last line", () => {
  assert.equal(summary(outcome), "markright: 1 issue(s)");
  assert.equal(summary({ issues: [], report: "", formatted: "" }), "");
});

test("lineRange finds a line, counted from 1", () => {
  const text = "one\ntwo\nthree";
  assert.deepEqual(lineRange(text, 1), [0, 3]);
  assert.deepEqual(lineRange(text, 2), [4, 7]);
  assert.deepEqual(lineRange(text, 3), [8, 13]);
  assert.deepEqual(lineRange(text, 9), [13, 13]);
});

test("debounce runs once, with the last arguments, after the calls stop", () => {
  mock.timers.enable({ apis: ["setTimeout"] });
  try {
    const seen: number[] = [];
    const later = debounce((n: number) => seen.push(n), 100);
    later(1);
    mock.timers.tick(50);
    later(2);
    mock.timers.tick(99);
    assert.deepEqual(seen, []);
    mock.timers.tick(1);
    assert.deepEqual(seen, [2]);
  } finally {
    mock.timers.reset();
  }
});

test("tabAfterKey follows the WAI-ARIA tabs pattern", () => {
  assert.equal(tabAfterKey("ArrowRight", 0, 2), 1);
  assert.equal(tabAfterKey("ArrowRight", 1, 2), 0);
  assert.equal(tabAfterKey("ArrowLeft", 0, 2), 1);
  assert.equal(tabAfterKey("ArrowLeft", 1, 3), 0);
  assert.equal(tabAfterKey("Home", 2, 3), 0);
  assert.equal(tabAfterKey("End", 0, 3), 2);
  assert.equal(tabAfterKey("Enter", 0, 3), undefined);
  assert.equal(tabAfterKey("toString", 0, 3), undefined);
});
