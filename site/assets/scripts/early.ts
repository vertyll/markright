// Runs in the head, before the page is drawn: it says the scripts run, and puts the reader's settings on the root
// element, so the stylesheets apply them from the first frame.
import { applyAttributes, createSettings, parseDefinitions } from "./lib/settings.ts";
import { browserStorage } from "./lib/storage.ts";

document.documentElement.classList.add("js");
const source = document.getElementById("markright-settings")?.textContent;
if (source) {
  applyAttributes(document.documentElement, createSettings(browserStorage(), parseDefinitions(source)));
}
