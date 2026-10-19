const EDITABLE_SELECTOR = "input, textarea, select, [contenteditable='true']";
const TEXT_CONTROL_SELECTOR = "input, textarea";

export function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(EDITABLE_SELECTOR) !== null;
}

/**
 * A control whose text, selection and edit history belong to the platform. A custom editable
 * surface such as a rich-text editor keeps its own history instead, so the two are not the same
 * question.
 */
export function isTextControlTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(TEXT_CONTROL_SELECTOR) !== null;
}
