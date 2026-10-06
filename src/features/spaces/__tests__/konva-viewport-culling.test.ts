import { describe, expect, it } from "vitest";

import { createSceneVisibilityIndex, visibleSceneIds } from "../canvas/konva/scene/viewport-culling";

describe("Konva viewport culling", () => {
  it("queries a reusable spatial index at translated and zoomed viewports", () => {
    const nodes = Array.from({ length: 10000 }, (_, id) => ({
      layout: { nodeId: String(id), frame: { x: id * 100, y: 0, width: 10, height: 10 } },
    }));
    const index = createSceneVisibilityIndex(nodes, [
      { key: "crossing", bounds: { x: 99950, y: 0, width: 200, height: 10 } },
      { key: "distant", bounds: { x: 0, y: 1000, width: 10, height: 10 } },
    ]);
    const visible = visibleSceneIds(index, { x: -200000, y: 0, zoom: 2 }, { width: 100, height: 100 }, 0);
    expect(visible.nodes).toEqual(new Set(["1000"]));
    expect(visible.relations).toEqual(new Set(["crossing"]));
    expect(visibleSceneIds(index, { x: 0, y: 0, zoom: 1 }, { width: 50, height: 50 }, 0).nodes).toEqual(
      new Set(["0"]),
    );
  });
  it("keeps intersecting nodes and excludes distant nodes", () => {
    const index = createSceneVisibilityIndex(
      [
        { layout: { nodeId: "inside", frame: { x: 20, y: 20, width: 100, height: 100 } } },
        { layout: { nodeId: "edge", frame: { x: 490, y: 390, width: 100, height: 100 } } },
        { layout: { nodeId: "outside", frame: { x: 900, y: 900, width: 100, height: 100 } } },
      ],
      [],
    );
    const visible = visibleSceneIds(index, { x: 0, y: 0, zoom: 1 }, { width: 500, height: 400 }, 0);

    expect(visible.nodes).toEqual(new Set(["inside", "edge"]));
  });
});
