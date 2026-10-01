import type { CanvasRect } from "../../contracts/canvas-editor-port";

export function selectionRectIntersects(left: CanvasRect, right: CanvasRect) {
  return (
    left.x <= right.x + right.width &&
    left.x + left.width >= right.x &&
    left.y <= right.y + right.height &&
    left.y + left.height >= right.y
  );
}
