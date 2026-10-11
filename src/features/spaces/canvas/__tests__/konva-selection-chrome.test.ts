import { describe, expect, it } from "vitest";

import { canvasChromeMetrics } from "../canvas-chrome-metrics";
import { connectorControlCenter } from "../konva/scene/connector-geometry";
import {
  CANVAS_FREE_RESIZE_ANCHORS,
  CANVAS_PROPORTIONAL_RESIZE_ANCHORS,
  CANVAS_TEXT_RESIZE_ANCHORS,
  isProportionalResizeAnchor,
} from "../konva/selection/canvas-resize-anchors";

describe("Konva selection chrome", () => {
  it("projects resize handles from each semantic resize mode", () => {
    expect(CANVAS_PROPORTIONAL_RESIZE_ANCHORS).toEqual([
      "top-left",
      "top-right",
      "bottom-left",
      "bottom-right",
    ]);
    expect(CANVAS_FREE_RESIZE_ANCHORS).toEqual([
      "top-left",
      "top-center",
      "top-right",
      "middle-right",
      "bottom-right",
      "bottom-center",
      "bottom-left",
      "middle-left",
    ]);
    expect(CANVAS_TEXT_RESIZE_ANCHORS).toEqual(["middle-right", "bottom-right", "middle-left"]);
    expect(isProportionalResizeAnchor("width-and-proportional", "bottom-right")).toBe(true);
    expect(isProportionalResizeAnchor("width-and-proportional", "middle-right")).toBe(false);
  });

  it("places connection controls outside every side of the selected node", () => {
    const frame = { x: 100, y: 200, width: 300, height: 180, rotation: 0 };

    expect(connectorControlCenter(frame, "top", 14)).toEqual({ x: 250, y: 186 });
    expect(connectorControlCenter(frame, "right", 14)).toEqual({ x: 414, y: 290 });
    expect(connectorControlCenter(frame, "bottom", 14)).toEqual({ x: 250, y: 394 });
    expect(connectorControlCenter(frame, "left", 14)).toEqual({ x: 86, y: 290 });
  });

  it("reduces resize-handle visual weight when the canvas zooms out", () => {
    const fullSize = canvasChromeMetrics(1).resize;
    const halfSize = canvasChromeMetrics(0.5).resize;
    const minimumZoomSize = canvasChromeMetrics(0.1).resize;

    expect(fullSize.size).toBe(9);
    expect(halfSize.size * 0.5).toBeCloseTo(9 * Math.sqrt(0.5));
    expect(minimumZoomSize.size * 0.1).toBeCloseTo(9 * Math.sqrt(0.1));
    expect(halfSize.hitStrokeWidth * 0.5).toBe(14);
  });
});
