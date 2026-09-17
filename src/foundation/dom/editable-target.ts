const EDITABLE_SELECTOR = "input, textarea, select, [contenteditable='true']";

export function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(EDITABLE_SELECTOR) !== null;
}
