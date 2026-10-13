/** Where settings are kept: localStorage in the browser, anything with the same two methods in tests. */
export type SettingStorage = Pick<Storage, "getItem" | "setItem">;

/** The browser's localStorage, or null where the page may not use it, as in some private windows. */
export function browserStorage(): Storage | null {
  try {
    return globalThis.localStorage;
  } catch {
    return null;
  }
}
