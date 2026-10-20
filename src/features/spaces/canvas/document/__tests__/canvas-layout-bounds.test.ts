import { describe, expect, it } from "vitest";

import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { canvasLayoutBounds } from "../canvas-layout-bounds";

describe("canvas layout bounds", () => {
  it("projects the outer bounds used by viewport navigation", () => {
    expect(canvasLayoutBounds([layout("a", [10, 20, 100, 80]), layout("b", [80, 60, 40, 30])])).toEqual({
      x: 10,
      y: 20,
      width: 110,
      height: 80,
    });
  });

  it("uses the rotated footprint for viewport navigation", () => {
    const rotated = layout("rotated", [900, 100, 100, 800]);
    rotated.frame.rotation = 90;
    expect(canvasLayoutBounds([rotated])).toEqual({
      x: 100,
      y: 100,
      width: 800,
      height: expect.closeTo(100, 10),
    });
  });

  it("has no bounds for an empty layout projection", () => {
    expect(canvasLayoutBounds([])).toBeUndefined();
  });
});

function layout(
  nodeId: string,
  [x, y, width, height]: readonly [number, number, number, number],
): CanvasNodeLayoutV1 {
  return { nodeId, frame: { x, y, width, height, rotation: 0 }, sizingMode: "frame", zIndex: 0 };
}
