/**
 * Where a canvas text editing session is present: the node it edits, the selection bar floating
 * above it, and every menu that bar opened. The bar and its menus are portalled out of the node and
 * take focus while they are used, so the editor losing focus says nothing about whether the user
 * left. A menu belongs to the surface of the trigger that labels it: the `aria-labelledby` an open
 * menu or popover carries from the moment it renders (its trigger's `aria-controls` is still empty
 * on a first open).
 */
export const CANVAS_TEXT_SURFACE_ATTRIBUTE = "data-canvas-text-surface";

export function isWithinCanvasTextSurface(nodeId: string, target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  if (surfaceOf(target) === nodeId) return true;
  const popup = target.closest("[role='menu'][aria-labelledby], [role='dialog'][aria-labelledby]");
  return (popup?.getAttribute("aria-labelledby") ?? "").split(/\s+/u).some((triggerId) => {
    const trigger = triggerId ? document.getElementById(triggerId) : null;
    return trigger !== null && surfaceOf(trigger) === nodeId;
  });
}

function surfaceOf(element: Element) {
  return element.closest(`[${CANVAS_TEXT_SURFACE_ATTRIBUTE}]`)?.getAttribute(CANVAS_TEXT_SURFACE_ATTRIBUTE);
}
