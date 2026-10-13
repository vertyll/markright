import { initNavigation } from "./components/navigation.ts";
import { initPlayground } from "./components/playground.ts";
import { initRuleLists } from "./components/rule-list.ts";
import { initSearch } from "./components/search.ts";
import { initSettings } from "./components/settings.ts";
import { initWrapToggles } from "./components/wrap-toggles.ts";
import { applyAttributes, createSettings, parseDefinitions } from "./lib/settings.ts";
import { browserStorage } from "./lib/storage.ts";

document.addEventListener("DOMContentLoaded", () => {
  const definitions = parseDefinitions(document.getElementById("markright-settings")?.textContent ?? "[]");
  const settings = createSettings(browserStorage(), definitions);
  settings.subscribe(() => applyAttributes(document.documentElement, settings));
  initNavigation();
  initSearch();
  initSettings(settings);
  initWrapToggles(settings);
  initRuleLists();
  initPlayground(settings);
});
