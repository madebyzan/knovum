const SCROLLABLE_SELECTOR = "[data-canvas-scrollable], .overflow-y-auto, .overflow-x-auto, .overflow-auto";

export function isScrollableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element) || target instanceof HTMLCanvasElement) return false;
  return target.closest(SCROLLABLE_SELECTOR) !== null;
}
