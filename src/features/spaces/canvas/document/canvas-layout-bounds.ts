import type { CanvasNodeLayoutV1 } from "../contract/canvas-document";

export function canvasLayoutBounds(layouts: readonly CanvasNodeLayoutV1[]) {
  if (!layouts.length) return undefined;
  const left = Math.min(...layouts.map(({ frame }) => frame.x));
  const top = Math.min(...layouts.map(({ frame }) => frame.y));
  const right = Math.max(...layouts.map(({ frame }) => frame.x + frame.width));
  const bottom = Math.max(...layouts.map(({ frame }) => frame.y + frame.height));
  return { x: left, y: top, width: right - left, height: bottom - top };
}
