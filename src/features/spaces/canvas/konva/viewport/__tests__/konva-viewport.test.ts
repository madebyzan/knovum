import { describe, expect, it } from "vitest";

import { canvasPointFromScreen, zoomViewportAt } from "../viewport-math";

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
});
