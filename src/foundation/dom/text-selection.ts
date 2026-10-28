import { isEditableTarget } from "./editable-target";

/**
 * Marks an element that owns editing while it holds focus, such as the canvas. A text selection left
 * behind somewhere else on the page is not what the user is acting on while such a surface has focus.
 */
const EDITING_SURFACE_ATTRIBUTE = "data-editing-surface";

function liveTextSelection(): Selection | undefined {
  const selection = document.getSelection();
  if (!selection || selection.isCollapsed || selection.rangeCount === 0) return undefined;
  return selection.toString() === "" ? undefined : selection;
}

/**
 * Whether the page holds a selection of read-only text that the user is acting on from where focus
 * is. Text can be selected anywhere on the page, not only inside an editable control, so the
 * clipboard has to follow the selection rather than the kind of element that holds focus.
 */
export function hasActionableTextSelection(focused: Element | null): boolean {
  const selection = liveTextSelection();
  if (!selection) return false;
  const surface = focused?.closest(`[${EDITING_SURFACE_ATTRIBUTE}]`);
  return !surface || surface.contains(selection.getRangeAt(0).commonAncestorContainer);
}

/** Whether a context menu opened on `target` belongs to the platform: it edits or copies that text. */
export function prefersSystemContextMenu(target: EventTarget | null): boolean {
  if (isEditableTarget(target)) return true;
  const selection = liveTextSelection();
  return !!selection && target instanceof Node && selection.getRangeAt(0).intersectsNode(target);
}
