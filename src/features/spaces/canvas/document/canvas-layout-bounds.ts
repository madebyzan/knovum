import { canvasFrameBounds } from "../canvas-snap-geometry";
import type { CanvasNodeLayoutV1 } from "../schema/document";

export function canvasLayoutBounds(layouts: readonly CanvasNodeLayoutV1[]) {
  let combined: Pick<CanvasNodeLayoutV1["frame"], "x" | "y" | "width" | "height"> | undefined;
  for (const { frame } of layouts) {
    const { left, top, width, height } = canvasFrameBounds(frame);
    const current = { x: left, y: top, width, height };
    if (!combined) {
      combined = current;
      continue;
    }
    const x = Math.min(combined.x, current.x);
    const y = Math.min(combined.y, current.y);
    combined = {
      x,
      y,
      width: Math.max(combined.x + combined.width, current.x + current.width) - x,
      height: Math.max(combined.y + combined.height, current.y + current.height) - y,
    };
  }
  return combined;
}
