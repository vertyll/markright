import { isRecord } from "./guards.ts";
import type { SettingStorage } from "./storage.ts";

/** One choice of a setting. */
export interface SettingOption {
  readonly value: string;
  readonly label: string;
}

/** A setting as data/settings.toml defines it. */
export interface SettingDefinition {
  readonly name: string;
  readonly default: string;
  readonly options: readonly SettingOption[];
}

/** Called whenever a setting changes. */
export type SettingListener = (name: string, value: string) => void;

/** The reader's settings, remembered between pages. */
export interface Settings {
  readonly definitions: readonly SettingDefinition[];
  /** The chosen value, or the default when nothing valid is stored. */
  get(name: string): string;
  /** Keeps value and tells the listeners; a name or value the definitions do not know is ignored. */
  set(name: string, value: string): void;
  /** Adds a listener and returns the function that removes it. */
  subscribe(listener: SettingListener): () => void;
}

/** The storage key of a setting. */
export const storageKey = (name: string): string => `markright-${name}`;

/** The attribute of the root element that carries a setting, for the stylesheets. */
export const attributeName = (name: string): string => `data-markright-${name}`;

/** Whether value is a setting's definition: a name, its options, and a default that is one of them. */
function isDefinition(value: unknown): value is SettingDefinition {
  if (!isRecord(value) || typeof value["name"] !== "string" || !Array.isArray(value["options"])) return false;
  const options: unknown[] = value["options"];
  const valid = options.every(
    (option) => isRecord(option) && typeof option["value"] === "string" && typeof option["label"] === "string",
  );
  return valid && options.some((option) => isRecord(option) && option["value"] === value["default"]);
}

/** Reads the definitions the page carries as JSON, and fails on anything that is not a list of them. */
export function parseDefinitions(raw: string): SettingDefinition[] {
  const value: unknown = JSON.parse(raw);
  if (!Array.isArray(value) || !value.every(isDefinition)) throw new TypeError("malformed settings");
  return value;
}

/**
 * Creates the settings over storage. Storage that is missing or throws reads as the defaults and still notifies the
 * listeners, so the page follows a choice until it is reloaded.
 */
export function createSettings(storage: SettingStorage | null, definitions: readonly SettingDefinition[]): Settings {
  const byName = new Map(definitions.map((definition) => [definition.name, definition]));
  const listeners = new Set<SettingListener>();
  const valid = (name: string, value: string | null) =>
    byName.get(name)?.options.some((option) => option.value === value);
  const get = (name: string): string => {
    let stored: string | null = null;
    try {
      stored = storage?.getItem(storageKey(name)) ?? null;
    } catch {
      // Read as the default.
    }
    return valid(name, stored) && stored !== null ? stored : (byName.get(name)?.default ?? "");
  };
  return {
    definitions,
    get,
    set(name, value) {
      if (!valid(name, value)) return;
      try {
        storage?.setItem(storageKey(name), value);
      } catch {
        // The choice still applies to this page.
      }
      for (const listener of listeners) listener(name, value);
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
  };
}

/** Sets every setting on element as its attribute. */
export function applyAttributes(element: Pick<Element, "setAttribute">, settings: Settings): void {
  for (const { name } of settings.definitions) element.setAttribute(attributeName(name), settings.get(name));
}
