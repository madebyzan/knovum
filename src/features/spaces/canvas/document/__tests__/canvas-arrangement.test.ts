import { describe, expect, it } from "vitest";

import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import {
  alignCanvasLayouts,
  distributeCanvasLayouts,
  fitSectionLayout,
} from "../canvas-arrangement";

describe("canvas arrangement", () => {
  it("aligns selected layouts without changing their dimensions", () => {
    const updates = alignCanvasLayouts(
      [layout("a", [10, 20, 100, 80]), layout("b", [80, 60, 40, 30])],
      "right",
    );

    expect(updates.map(({ frame }) => [frame.x, frame.width])).toEqual([
      [20, 100],
      [80, 40],
    ]);
  });

  it("distributes three layouts across their current outer bounds", () => {
    const updates = distributeCanvasLayouts(
      [layout("a", [0, 0, 20, 20]), layout("b", [30, 0, 20, 20]), layout("c", [100, 0, 20, 20])],
      "horizontal",
    );

    expect(updates.map(({ frame }) => frame.x)).toEqual([0, 50, 100]);
  });

  it("fits a section around its members with desktop canvas spacing", () => {
    const [update] = fitSectionLayout(layout("section", [0, 0, 200, 200]), [
      layout("a", [100, 120, 80, 60]),
      layout("b", [240, 220, 100, 80]),
    ]);

    expect(update?.frame).toEqual({ x: 68, y: 64, width: 304, height: 268, rotation: 0 });
  });
});

function layout(
  nodeId: string,
  [x, y, width, height]: readonly [number, number, number, number],
): CanvasNodeLayoutV1 {
  return { nodeId, frame: { x, y, width, height, rotation: 0 }, sizeAuthority: "frame", zIndex: 0 };
}
