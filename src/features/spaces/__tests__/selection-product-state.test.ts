import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { canvasSelectionProductState } from "../presentation/canvas/selection-product-state";

const nodes = ["a", "b"].map((id) => createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id));

describe("canvasSelectionProductState", () => {
  it("connects exactly two explicitly selected semantic content items", () => {
    expect(canvasSelectionProductState(nodes, ["a", "b"], [], snapshot()).canConnect).toBe(true);
    expect(canvasSelectionProductState(nodes, [], ["section"], snapshot()).canConnect).toBe(false);
  });

  it("projects Section selection and ancestor locks from Structure", () => {
    const state = canvasSelectionProductState(nodes, [], ["section"], snapshot(true));

    expect(state.section).toEqual({ id: "section", collapsed: true });
    expect(state.content.map(({ id }) => id)).toEqual(["a", "b"]);
    expect(state.allLocked).toBe(true);
    expect(state.canCreateGroup).toBe(false);
  });
});

function snapshot(locked = false): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked, title: "Research" },
      { id: "item-a", kind: "content", parentId: "section", order: 0, locked: false, nodeId: "a" },
      { id: "item-b", kind: "content", parentId: "section", order: 1, locked: false, nodeId: "b" },
    ],
    layouts: [
      {
        structureNodeId: "section",
        kind: "section",
        position: { x: 0, y: 0 },
        size: { width: 400, height: 300 },
        collapsed: true,
      },
      {
        structureNodeId: "item-a",
        kind: "content",
        transform: { x: 20, y: 60, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
      {
        structureNodeId: "item-b",
        kind: "content",
        transform: { x: 140, y: 60, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
    ],
  };
}
