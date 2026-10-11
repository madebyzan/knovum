/** Owns DOM pointer capture for one canvas interaction at a time. */
export function createCanvasPointerCapture() {
  let captured: { element: Element; id: number } | undefined;
  function capture(event: PointerEvent) {
    const element = event.currentTarget;
    if (
      !(element instanceof Element) ||
      !element.hasPointerCapture ||
      element.hasPointerCapture(event.pointerId)
    )
      return;
    element.setPointerCapture(event.pointerId);
    captured = { element, id: event.pointerId };
  }
  function release() {
    if (!captured) return;
    const { element, id } = captured;
    captured = undefined;
    if (element.hasPointerCapture(id)) element.releasePointerCapture(id);
  }
  return { capture, release, accepts: (pointerId: number) => !captured || captured.id === pointerId };
}
