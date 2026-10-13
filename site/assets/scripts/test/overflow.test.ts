import assert from "node:assert/strict";
import { test } from "node:test";
import { overflowsUnwrapped, type Scroller, type WrapHost } from "../lib/overflow.ts";

function host(wrapped: boolean): WrapHost & { classes: Set<string> } {
  const classes = new Set(wrapped ? ["wrapped"] : []);
  return {
    classes,
    classList: {
      contains: (name) => classes.has(name),
      remove: (...names) => {
        for (const name of names) classes.delete(name);
      },
      toggle: (name, on) => {
        if (on) classes.add(name);
        else classes.delete(name);
        return Boolean(on);
      },
    },
  };
}

function scroller(h: { classes: Set<string> }, unwrapped: number, wrapped: number, width: number): Scroller {
  return {
    get scrollWidth() {
      return h.classes.has("wrapped") ? wrapped : unwrapped;
    },
    clientWidth: width,
  };
}

test("measures without wrapping, so a wrapped block still knows it is wide", () => {
  const h = host(true);
  assert.equal(overflowsUnwrapped(h, scroller(h, 900, 400, 400), "wrapped"), true);
  assert.equal(h.classes.has("wrapped"), true);
});

test("a block that fits reports no overflow and keeps its state", () => {
  const h = host(false);
  assert.equal(overflowsUnwrapped(h, scroller(h, 300, 300, 400), "wrapped"), false);
  assert.equal(h.classes.has("wrapped"), false);
});
