import { describe, expect, it } from "vitest";

import type { CanvasNodeLayoutV1 } from "../canvas/contract/canvas-document";
import { sectionAssignments } from "../canvas/konva/scene/section-membership";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

describe("canvas section membership", () => {
  it("uses a collapsed section's visible bar instead of its stored expanded bounds", () => {
    const section: SpaceCanvasNode = {
      id: "section",
      type: "section",
      data: { kind: "section", title: "Section", collapsed: true },
      initialPosition: { x: 0, y: 0 },
    };
    const note: SpaceCanvasNode = {
      id: "note",
      type: "note",
      data: createSpaceNotePayload("Note"),
      initialPosition: { x: 100, y: 300 },
    };
    const layouts: CanvasNodeLayoutV1[] = [
      layout("section", [0, 0, 640, 420]),
      layout("note", [100, 300, 300, 220]),
    ];

    expect(sectionAssignments(["note"], layouts, new Set(["section"]), [section, note])).toEqual([]);
  });
});

function layout(nodeId: string, [x, y, width, height]: readonly [number, number, number, number]) {
  return { nodeId, frame: { x, y, width, height, rotation: 0 }, sizeAuthority: "frame" as const, zIndex: 0 };
}
