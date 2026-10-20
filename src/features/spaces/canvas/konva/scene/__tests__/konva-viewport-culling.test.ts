import { describe, expect, it } from "vitest";

import { createSceneVisibilityIndex, sceneOverscan, visibleSceneIds } from "../viewport-culling";

const NO_OVERSCAN = { bottom: 0, left: 0, right: 0, top: 0 };

describe("Konva viewport culling", () => {
  it("queries a reusable spatial index at translated and zoomed viewports", () => {
    const nodes = Array.from({ length: 10000 }, (_, id) => ({
      layout: { nodeId: String(id), frame: { x: id * 100, y: 0, width: 10, height: 10, rotation: 0 } },
    }));
    const index = createSceneVisibilityIndex(
      nodes,
      [
        { key: "crossing", bounds: { x: 99950, y: 0, width: 200, height: 10 } },
        { key: "distant", bounds: { x: 0, y: 1000, width: 10, height: 10 } },
      ],
      [],
    );
    const visible = visibleSceneIds(
      index,
      { x: -200000, y: 0, zoom: 2 },
      { width: 100, height: 100 },
      NO_OVERSCAN,
    );
    expect(visible.nodes).toEqual(new Set(["1000"]));
    expect(visible.relations).toEqual(new Set(["crossing"]));
    expect(
      visibleSceneIds(index, { x: 0, y: 0, zoom: 1 }, { width: 50, height: 50 }, NO_OVERSCAN).nodes,
    ).toEqual(new Set(["0"]));
  });
  it("keeps intersecting nodes and excludes distant nodes", () => {
    const index = createSceneVisibilityIndex(
      [
        { layout: { nodeId: "inside", frame: { x: 20, y: 20, width: 100, height: 100, rotation: 0 } } },
        { layout: { nodeId: "edge", frame: { x: 490, y: 390, width: 100, height: 100, rotation: 0 } } },
        { layout: { nodeId: "outside", frame: { x: 900, y: 900, width: 100, height: 100, rotation: 0 } } },
      ],
      [],
      [],
    );
    const visible = visibleSceneIds(index, { x: 0, y: 0, zoom: 1 }, { width: 500, height: 400 }, NO_OVERSCAN);

    expect(visible.nodes).toEqual(new Set(["inside", "edge"]));
  });
  it("retains the visible rotated footprint rather than the unrotated frame", () => {
    const index = createSceneVisibilityIndex(
      [
        { layout: { nodeId: "rotated", frame: { x: 900, y: 100, width: 100, height: 800, rotation: 90 } } },
        { layout: { nodeId: "plain", frame: { x: 900, y: 100, width: 100, height: 800, rotation: 0 } } },
      ],
      [],
      [],
    );
    expect(
      visibleSceneIds(index, { x: 0, y: 0, zoom: 1 }, { width: 500, height: 400 }, NO_OVERSCAN).nodes,
    ).toEqual(new Set(["rotated"]));
  });
  it("queries section backgrounds and interaction chrome through the spatial index", () => {
    const index = createSceneVisibilityIndex(
      [],
      [],
      [
        { key: "inside", bounds: { x: 20, y: 20, width: 100, height: 100 } },
        { key: "outside", bounds: { x: 900, y: 900, width: 100, height: 100 } },
      ],
    );
    expect(
      visibleSceneIds(index, { x: 0, y: 0, zoom: 1 }, { width: 500, height: 400 }, NO_OVERSCAN).sections,
    ).toEqual(new Set(["inside"]));
  });
  it("runs the overscan ahead of the travelling camera instead of behind it", () => {
    const index = createSceneVisibilityIndex(
      [
        { layout: { nodeId: "ahead", frame: { x: -900, y: 0, width: 100, height: 100, rotation: 0 } } },
        { layout: { nodeId: "behind", frame: { x: 900, y: 0, width: 100, height: 100, rotation: 0 } } },
      ],
      [],
      [],
    );
    const size = { width: 500, height: 400 };
    const viewport = { x: 0, y: 0, zoom: 1 };
    // A rising stage position travels towards smaller world coordinates, so "ahead" is the node at -900.
    const travelling = visibleSceneIds(index, viewport, size, sceneOverscan({ x: 400, y: 0 }));
    expect(travelling.nodes).toEqual(new Set(["ahead"]));
    expect(visibleSceneIds(index, viewport, size, sceneOverscan({ x: -400, y: 0 })).nodes).toEqual(
      new Set(["behind"]),
    );
    expect(visibleSceneIds(index, viewport, size, sceneOverscan({ x: 0, y: 0 })).nodes).toEqual(new Set());
  });
  it("clamps the lead so a flung camera cannot widen the query without bound", () => {
    expect(sceneOverscan({ x: 100_000, y: 0 })).toEqual({ bottom: 240, left: 1200, right: 240, top: 240 });
  });
});
