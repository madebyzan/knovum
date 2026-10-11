import { describe, expect, it } from "vitest";

import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { projectStructureCanvasDocument } from "../document/structure-canvas-document";
import { projectStructureGesture, resizedSectionFrame } from "../konva/interaction/structure-gesture-preview";

describe("structure gesture preview", () => {
  it("moves the Section and nested Group content together without changing membership", () => {
    const before = snapshot();
    const after = projectStructureGesture(before, {
      kind: "structure.move",
      rootIds: ["section"],
      delta: { x: 70, y: -20 },
    });
    const initial = projectStructureCanvasDocument(before).nodes[0]!.frame;
    expect(projectStructureCanvasDocument(after).nodes[0]!.frame).toEqual({
      ...initial,
      x: initial.x + 70,
      y: initial.y - 20,
    });
    expect(after.structure).toBe(before.structure);
    expect(before.layouts[0]).toMatchObject({ position: { x: 10, y: 20 } });
  });

  it("keeps descendants fixed in world space when resizing from the north-west", () => {
    const before = snapshot();
    const after = projectStructureGesture(before, {
      kind: "section.resize-frame",
      structureNodeId: "section",
      frame: { x: -60, y: -40, width: 470, height: 360 },
    });
    expect(projectStructureCanvasDocument(after).nodes).toEqual(projectStructureCanvasDocument(before).nodes);
    expect(after.layouts[0]).toMatchObject({
      position: { x: -60, y: -40 },
      size: { width: 470, height: 360 },
    });
  });

  it("translates a root inside a rotated scaled Group in world coordinates", () => {
    const before = snapshot();
    const after = projectStructureGesture(before, {
      kind: "structure.move",
      rootIds: ["content"],
      delta: { x: 40, y: 60 },
    });
    const initial = projectStructureCanvasDocument(before).nodes[0]!.frame;
    expect(projectStructureCanvasDocument(after).nodes[0]!.frame).toEqual({
      ...initial,
      x: initial.x + 40,
      y: initial.y + 60,
    });
  });

  it("clamps north-west resizing without moving the opposite corner", () => {
    expect(
      resizedSectionFrame({ x: 10, y: 20, width: 400, height: 300 }, "north-west", { x: 300, y: 250 }),
    ).toEqual({ x: 170, y: 160, width: 240, height: 160 });
  });
});

function snapshot(): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Sources" },
      { id: "group", kind: "group", parentId: "section", order: 0, locked: false, title: "Nested" },
      { id: "content", kind: "content", parentId: "group", order: 0, locked: false, nodeId: "note" },
    ],
    layouts: [
      {
        structureNodeId: "section",
        kind: "section",
        position: { x: 10, y: 20 },
        size: { width: 400, height: 300 },
        collapsed: false,
      },
      { structureNodeId: "group", kind: "group", transform: { x: 30, y: 60, rotation: 90, scale: 2 } },
      {
        structureNodeId: "content",
        kind: "content",
        transform: { x: 10, y: 20, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
    ],
  };
}
