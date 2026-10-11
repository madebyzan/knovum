import { describe, expect, it } from "vitest";

import { fitCanvasObjects } from "../canvas/konva/viewport/canvas-editor-navigation";
import { canvasPointFromScreen, zoomViewportAt } from "../canvas/konva/viewport/viewport-math";

describe("Konva canvas viewport", () => {
  it("keeps the canvas point under the pointer stable while zooming", () => {
    const viewport = { x: 120, y: 80, zoom: 1 };
    const pointer = { x: 320, y: 280 };
    const before = canvasPointFromScreen(pointer, viewport);
    const next = zoomViewportAt(viewport, pointer, 2);

    expect(canvasPointFromScreen(pointer, next)).toEqual(before);
  });

  it("projects screen coordinates into canvas coordinates", () => {
    expect(canvasPointFromScreen({ x: 250, y: 180 }, { x: 50, y: 30, zoom: 2 })).toEqual({
      x: 100,
      y: 75,
    });
  });
  it("fits an empty Section even when no content node layout exists", () => {
    let viewport = { x: 0, y: 0, zoom: 1 };
    fitCanvasObjects(
      {
        setViewport: (next) => {
          viewport = next;
        },
        size: () => ({ width: 1000, height: 800 }),
        viewport: () => viewport,
      },
      [{ x: 2000, y: 1000, width: 400, height: 300 }],
    );
    expect(viewport).toEqual({ x: -3900, y: -1900, zoom: 2 });
  });
});
