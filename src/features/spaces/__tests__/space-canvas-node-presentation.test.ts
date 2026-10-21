import { describe, expect, it } from "vitest";

import { canvasRelationArrowheads } from "../canvas/konva/rendering/connector-presentation";
import type { CanvasSceneRelation } from "../canvas/konva/scene/canvas-scene";
import { spaceCanvasNodePresentation } from "../canvas/presentation/nodes/space-canvas-node-presentation";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";

describe("Konva node presentation", () => {
  it("keeps type-specific node content", () => {
    const note = spaceCanvasNodePresentation(createSpaceNotePayload("Note", "Body"));
    const website = spaceCanvasNodePresentation({
      kind: "website",
      title: "Knovum",
      url: "https://example.com",
      siteName: "Example",
    });

    expect(note).toMatchObject({ title: "Note", body: "Body", subtitle: "Note" });
    expect(website).toMatchObject({ title: "Knovum", subtitle: "Example" });
  });

  it("does not invent direction for symmetric semantic relations", () => {
    const relation: CanvasSceneRelation = {
      id: "edge",
      key: "relation:edge",
      kind: "relation",
      label: "related",
      color: "neutral",
      bounds: { x: 0, y: 0, width: 100, height: 100 },
      midpoint: { x: 50, y: 50 },
      points: [0, 0, 30, 0, 70, 100, 100, 100],
      sourceNodeId: "a",
      sourceFrame: { x: 0, y: 0, width: 100, height: 80, rotation: 0 },
      targetNodeId: "b",
      targetFrame: { x: 100, y: 100, width: 100, height: 80, rotation: 0 },
      parallelOffset: 0,
      lineage: false,
      relations: [
        {
          id: "edge",
          spaceId: "space",
          sourceId: "a",
          targetId: "b",
          kind: "related",
          rationale: "",
          status: "active",
          actorKind: "human",
          createdAt: 1,
          updatedAt: 1,
        },
      ],
    };

    expect(canvasRelationArrowheads(relation)).toEqual({ source: false, target: false });
    expect(canvasRelationArrowheads({ ...relation, label: "supports" })).toEqual({
      source: false,
      target: true,
    });
  });
});
