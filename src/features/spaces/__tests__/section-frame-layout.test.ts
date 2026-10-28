import { describe, expect, it } from "vitest";

import { editedSectionSize, sectionSizePreset } from "../canvas/presentation/selection/section-frame-size";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { arrangementAvailability } from "../structure/action-availability";

describe("Section frame size", () => {
  it("names a preset only for its exact size", () => {
    expect(sectionSizePreset({ width: 1920, height: 1080 })).toBe("16:9");
    expect(sectionSizePreset({ width: 1920, height: 1081 })).toBeUndefined();
  });

  it("edits one dimension freely or carries the other with a locked ratio", () => {
    const size = { width: 1600, height: 900 };
    expect(editedSectionSize(size, { width: 800 }, false)).toEqual({ width: 800, height: 900 });
    expect(editedSectionSize(size, { width: 800 }, true)).toEqual({ width: 800, height: 450 });
    expect(editedSectionSize(size, { height: 1800 }, true)).toEqual({ width: 3200, height: 1800 });
  });

  it("keeps the Section minimum without breaking a locked ratio", () => {
    expect(editedSectionSize({ width: 1600, height: 900 }, { width: 10 }, false)).toEqual({
      width: 240,
      height: 900,
    });
    expect(editedSectionSize({ width: 1600, height: 900 }, { width: 100 }, true)).toEqual({
      width: 284,
      height: 160,
    });
  });
});

describe("arrangementAvailability", () => {
  it("scales with the selection and needs one parent to lay out", () => {
    expect(arrangementAvailability(snapshot(false), ["a"])).toEqual({
      align: false,
      distribute: false,
      autoLayout: false,
    });
    expect(arrangementAvailability(snapshot(false), ["a", "b"])).toEqual({
      align: true,
      distribute: false,
      autoLayout: true,
    });
    expect(arrangementAvailability(snapshot(false), ["a", "b", "c"])).toEqual({
      align: true,
      distribute: true,
      autoLayout: false,
    });
  });

  it("leaves the children of an auto-layout Section to the Section", () => {
    expect(arrangementAvailability(snapshot(true), ["a", "b"])).toEqual({
      align: false,
      distribute: false,
      autoLayout: false,
    });
  });
});

function content(id: string, parentId: string | null, order: number) {
  return {
    node: { id, kind: "content" as const, parentId, order, locked: false, nodeId: `n${id}` },
    layout: {
      structureNodeId: id,
      kind: "content" as const,
      transform: { x: order * 120, y: 60, rotation: 0, scale: 1 },
      size: { width: 100, height: 80 },
      sizingMode: "frame" as const,
    },
  };
}

function snapshot(autoLayout: boolean): SpaceEditSnapshot {
  const items = [content("a", "frame", 0), content("b", "frame", 1), content("c", null, 1)];
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "frame", kind: "section", parentId: null, order: 0, locked: false, title: "Frame" },
      ...items.map(({ node }) => node),
    ],
    layouts: [
      {
        structureNodeId: "frame",
        kind: "section",
        position: { x: 0, y: 0 },
        size: { width: 640, height: 400 },
        collapsed: false,
        autoLayout,
      },
      ...items.map(({ layout }) => layout),
    ],
  };
}
