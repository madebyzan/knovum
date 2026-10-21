const EDITABLE_SELECTOR = "input, textarea, select, [contenteditable='true']";
const RICH_TEXT_SELECTOR = "[contenteditable='true']";
/** Input types that take no typed text, so they hold no text selection and no edit history. */
const NON_TEXT_INPUT_TYPES = new Set([
  "button",
  "checkbox",
  "color",
  "file",
  "hidden",
  "image",
  "radio",
  "range",
  "reset",
  "submit",
]);

export function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(EDITABLE_SELECTOR) !== null;
}

/**
 * A native field the user types into. Its text, selection and edit history all belong to the
 * platform.
 */
export function isTextFieldTarget(target: EventTarget | null): boolean {
  if (target instanceof HTMLTextAreaElement) return true;
  return target instanceof HTMLInputElement && !NON_TEXT_INPUT_TYPES.has(target.type);
}

/**
 * Editable rich text. The platform still owns its clipboard and selection, but the editor behind it
 * keeps an edit history of its own.
 */
export function isRichTextTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(RICH_TEXT_SELECTOR) !== null;
}
