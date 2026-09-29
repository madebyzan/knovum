/** Inverts the canvas overlay's uniform zoom and node rotation for DOM pointer gestures. */
export function viewportToLocalPoint(
  element: HTMLElement,
  clientX: number,
  clientY: number,
  rotation: number,
) {
  const { left, top, width, height } = element.getBoundingClientRect();
  const localWidth = element.offsetWidth;
  const localHeight = element.offsetHeight;
  const radians = (rotation * Math.PI) / 180;
  const cosine = Math.cos(radians);
  const sine = Math.sin(radians);
  const rotatedWidth = Math.abs(localWidth * cosine) + Math.abs(localHeight * sine);
  const zoom = width / rotatedWidth;
  if (!Number.isFinite(zoom) || zoom <= 0 || height <= 0)
    throw new Error("Canvas pointer target has no size");
  const minX = Math.min(
    0,
    localWidth * cosine,
    -localHeight * sine,
    localWidth * cosine - localHeight * sine,
  );
  const minY = Math.min(0, localWidth * sine, localHeight * cosine, localWidth * sine + localHeight * cosine);
  const x = (clientX - left + minX * zoom) / zoom;
  const y = (clientY - top + minY * zoom) / zoom;
  return { x: x * cosine + y * sine, y: -x * sine + y * cosine };
}
