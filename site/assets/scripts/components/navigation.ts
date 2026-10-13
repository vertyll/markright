/** A button that shows and hides what it controls, by putting the open class on it. */
function initDisclosure(button: HTMLButtonElement | null, target: HTMLElement | null): void {
  if (!button || !target) return;
  button.addEventListener("click", () => {
    const open = target.classList.toggle("open");
    button.setAttribute("aria-expanded", String(open));
  });
}

/** The menu button of narrow screens, and the button that folds the documentation's navigation. */
export function initNavigation(): void {
  initDisclosure(document.querySelector<HTMLButtonElement>("[data-menu-toggle]"), document.getElementById("site-nav"));
  initDisclosure(
    document.querySelector<HTMLButtonElement>("[data-docs-nav-toggle]"),
    document.querySelector<HTMLElement>("[data-docs-nav]"),
  );
}
