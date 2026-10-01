import type { CanvasPoint, CanvasViewport } from "../../contracts/canvas-editor-port";

export function canvasPastePoint(
  pointer: CanvasPoint | undefined,
  viewport: CanvasViewport,
  size: { width: number; height: number },
) {
  return (
    pointer ?? {
      x: (size.width / 2 - viewport.x) / viewport.zoom,
      y: (size.height / 2 - viewport.y) / viewport.zoom,
    }
  );
}
