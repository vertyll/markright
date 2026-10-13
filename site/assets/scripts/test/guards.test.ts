import assert from "node:assert/strict";
import { test } from "node:test";
import { isRecord } from "../lib/guards.ts";

test("isRecord accepts objects and arrays, nothing else", () => {
  assert.ok(isRecord({}) && isRecord({ a: 1 }) && isRecord([]));
  for (const value of [null, undefined, 1, "x", true]) assert.ok(!isRecord(value), String(value));
});
