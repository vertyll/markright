/** The element whose class decides whether its content wraps. */
export interface WrapHost {
  readonly classList: Pick<DOMTokenList, "contains" | "remove" | "toggle">;
}

/** The element that scrolls when its content does not wrap. */
export interface Scroller {
  readonly scrollWidth: number;
  readonly clientWidth: number;
}

/**
 * Whether the content is wider than the scroller with wrapping off. It measures without the wrapped class and puts
 * the class back, so a block that is wrapped now still knows it needs the button.
 */
export function overflowsUnwrapped(host: WrapHost, scroller: Scroller, wrappedClass: string): boolean {
  const wrapped = host.classList.contains(wrappedClass);
  host.classList.remove(wrappedClass);
  const wide = scroller.scrollWidth > scroller.clientWidth;
  host.classList.toggle(wrappedClass, wrapped);
  return wide;
}
