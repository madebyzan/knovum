const INTERACTIVE_SELECTOR = [
  "button",
  "a",
  "[role='button']",
  "[role='menuitem']",
  "[role='option']",
  "[role='slider']",
  "[role='checkbox']",
  "[role='tab']",
  "[role='listbox']",
  "[data-bare-key-scope]",
].join(",");

/**
 * A focused control or embedded surface that gives its own meaning to bare key presses
 * (such as Backspace, Delete, Space, Escape, or navigation keys) so canvas shortcuts do not hijack them.
 */
export function isInteractiveTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(INTERACTIVE_SELECTOR) !== null;
}
