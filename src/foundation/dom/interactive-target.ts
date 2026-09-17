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
].join(",");

/** A focused control the platform already gives meaning to a bare key press, such as Space. */
export function isInteractiveTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(INTERACTIVE_SELECTOR) !== null;
}
