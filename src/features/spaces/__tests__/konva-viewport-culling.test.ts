import { describe, expect, it } from "vitest";

import { visibleSceneNodeIds } from "../canvas/konva/scene/viewport-culling";

describe("Konva viewport culling", () => {
  it("keeps intersecting nodes and excludes distant nodes", () => {
    const visible = visibleSceneNodeIds(
      [
        { layout: { nodeId: "inside", frame: { x: 20, y: 20, width: 100, height: 100 } } },
        { layout: { nodeId: "edge", frame: { x: 490, y: 390, width: 100, height: 100 } } },
        { layout: { nodeId: "outside", frame: { x: 900, y: 900, width: 100, height: 100 } } },
      ],
      { x: 0, y: 0, zoom: 1 },
      { width: 500, height: 400 },
      0,
    );

    expect(visible).toEqual(new Set(["inside", "edge"]));
  });
});
