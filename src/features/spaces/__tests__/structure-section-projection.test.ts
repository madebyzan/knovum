import { describe, expect, it } from "vitest";

import { projectStructureCanvasDocument } from "../canvas/document/structure-canvas-document";
import { projectStructureScene } from "../canvas/document/structure-scene-projection";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { sectionDropCandidateId, sectionDropReparentCommand } from "../structure/section-drop-reparent";

describe("Structure Section projection", () => {
  it("keeps folded descendants in the adapter document while hiding only the scene projection", () => {
    const snapshot = collapsedSectionSnapshot();
    const node = createSpaceNode(createSpaceNotePayload("Evidence"), { x: 0, y: 0 }, "node");

    expect(projectStructureCanvasDocument(snapshot).nodes.map(({ nodeId }) => nodeId)).toEqual(["node"]);
    expect(projectStructureScene({ snapshot, nodes: [node], relations: [] }).content[0]?.visible).toBe(false);
    expect(
      projectStructureScene({
        snapshot,
        nodes: [node],
        relations: [],
        temporarilyExpandedSectionIds: new Set(["section"]),
      }).content[0]?.visible,
    ).toBe(true);
  });

  it("requires an explicitly expanded collapsed Section before producing a drop command", () => {
    const snapshot = dropSnapshot();
    const delta = { x: -350, y: 0 };

    expect(sectionDropCandidateId(snapshot, ["free"], delta)).toBeUndefined();
    expect(sectionDropCandidateId(snapshot, ["free"], delta, true)).toBe("section");
    expect(sectionDropReparentCommand(snapshot, ["free"], delta)).toBeUndefined();
    expect(
      sectionDropReparentCommand(snapshot, ["free"], delta, {
        allowCollapsedSectionId: "section",
      }),
    ).toEqual({ kind: "structure.reparent", rootIds: ["free"], parentId: "section", order: 0 });
  });
});

function collapsedSectionSnapshot(): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Sources" },
      { id: "content", kind: "content", parentId: "section", order: 0, locked: false, nodeId: "node" },
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
        structureNodeId: "content",
        kind: "content",
        transform: { x: 40, y: 70, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
    ],
  };
}

function dropSnapshot(): SpaceEditSnapshot {
  const snapshot = collapsedSectionSnapshot();
  return {
    ...snapshot,
    structure: [
      snapshot.structure[0]!,
      { id: "free", kind: "content", parentId: null, order: 1, locked: false, nodeId: "free-node" },
    ],
    layouts: [
      snapshot.layouts[0]!,
      {
        structureNodeId: "free",
        kind: "content",
        transform: { x: 500, y: 100, rotation: 0, scale: 1 },
        size: { width: 80, height: 60 },
        sizingMode: "frame",
      },
    ],
  };
}
