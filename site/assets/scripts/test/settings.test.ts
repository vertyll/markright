import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applyAttributes,
  attributeName,
  createSettings,
  parseDefinitions,
  type SettingDefinition,
  storageKey,
} from "../lib/settings.ts";
import type { SettingStorage } from "../lib/storage.ts";

const definitions: SettingDefinition[] = [
  {
    name: "code",
    default: "scroll",
    options: [
      { value: "scroll", label: "Scroll" },
      { value: "wrap", label: "Wrap" },
    ],
  },
  {
    name: "text",
    default: "normal",
    options: [
      { value: "normal", label: "Default" },
      { value: "large", label: "Large" },
    ],
  },
];

function memoryStorage(): SettingStorage & { data: Map<string, string> } {
  const data = new Map<string, string>();
  return { getItem: (key) => data.get(key) ?? null, setItem: (key, value) => void data.set(key, value), data };
}

test("a setting is its default until it is set", () => {
  const settings = createSettings(memoryStorage(), definitions);
  assert.equal(settings.get("code"), "scroll");
  assert.equal(settings.get("text"), "normal");
  assert.equal(settings.get("unknown"), "");
});

test("a choice is kept under its own key and read back", () => {
  const storage = memoryStorage();
  createSettings(storage, definitions).set("text", "large");
  assert.equal(storage.data.get(storageKey("text")), "large");
  assert.equal(createSettings(storage, definitions).get("text"), "large");
  assert.equal(createSettings(storage, definitions).get("code"), "scroll");
});

test("unknown names, unknown values and stale stored values are ignored", () => {
  const storage = memoryStorage();
  storage.data.set(storageKey("code"), "sideways");
  const settings = createSettings(storage, definitions);
  const heard: string[] = [];
  settings.subscribe((name, value) => heard.push(`${name}=${value}`));
  settings.set("code", "diagonal");
  settings.set("colour", "red");
  assert.equal(settings.get("code"), "scroll");
  assert.deepEqual(heard, []);
});

test("listeners hear every change until they unsubscribe", () => {
  const settings = createSettings(memoryStorage(), definitions);
  const heard: string[] = [];
  const stop = settings.subscribe((name, value) => heard.push(`${name}=${value}`));
  settings.set("code", "wrap");
  stop();
  settings.set("code", "scroll");
  assert.deepEqual(heard, ["code=wrap"]);
});

test("storage that throws or is missing reads as the defaults and still notifies", () => {
  const blocked = (): never => {
    throw new Error("blocked");
  };
  for (const storage of [{ getItem: blocked, setItem: blocked }, null]) {
    const settings = createSettings(storage, definitions);
    let heard = false;
    settings.subscribe(() => (heard = true));
    settings.set("code", "wrap");
    assert.equal(settings.get("code"), "scroll");
    assert.equal(heard, true);
  }
});

test("applyAttributes puts every setting on the element", () => {
  const storage = memoryStorage();
  storage.data.set(storageKey("text"), "large");
  const attributes = new Map<string, string>();
  applyAttributes(
    { setAttribute: (name, value) => void attributes.set(name, value) },
    createSettings(storage, definitions),
  );
  assert.deepEqual(Object.fromEntries(attributes), {
    [attributeName("code")]: "scroll",
    [attributeName("text")]: "large",
  });
});

test("parseDefinitions reads the page's definitions and rejects anything else", () => {
  assert.deepEqual(parseDefinitions(JSON.stringify(definitions)), definitions);
  for (const raw of [
    "{}",
    "[1]",
    '[{"name":"x","default":"a","options":[{"value":"b","label":"B"}]}]',
    '[{"name":"x"}]',
  ]) {
    assert.throws(() => parseDefinitions(raw), TypeError, raw);
  }
});
