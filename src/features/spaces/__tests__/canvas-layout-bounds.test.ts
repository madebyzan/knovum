import { describe, expect, it } from "vitest";

import { canvasLayoutBounds } from "../canvas/document/canvas-layout-bounds";
import type { CanvasNodeLayoutV1 } from "../canvas/schema/document";

describe("canvas layout bounds", () => {
  it("projects the outer bounds used by viewport navigation", () => {
    expect(canvasLayoutBounds([layout("a", [10, 20, 100, 80]), layout("b", [80, 60, 40, 30])])).toEqual({
      x: 10,
      y: 20,
      width: 110,
      height: 80,
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
