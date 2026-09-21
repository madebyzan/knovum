import { describe, expect, it } from "vitest";

import type { CanvasSceneRelation } from "../../scene/canvas-scene";
import { canvasRelationArrowheads } from "../connector-presentation";

describe("connector presentation", () => {
  it("does not invent direction for symmetric semantic relations", () => {
    const relation: CanvasSceneRelation = {
      id: "edge",
      key: "relation:edge",
      kind: "relation",
      label: "related",
      color: "neutral",
      bounds: { x: 0, y: 0, width: 100, height: 100 },
      midpoint: { x: 50, y: 50 },
      sourceNodeId: "a",
      targetNodeId: "b",
      parallelOffset: 0,
      points: [0, 0, 30, 0, 70, 100, 100, 100],
      relation: {
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
    };

    expect(canvasRelationArrowheads(relation)).toEqual({ source: false, target: false });
    expect(
      canvasRelationArrowheads({ ...relation, relation: { ...relation.relation, kind: "supports" } }),
    ).toEqual({ source: false, target: true });
  });
});
