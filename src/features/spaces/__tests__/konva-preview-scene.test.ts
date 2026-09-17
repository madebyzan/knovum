import { describe, expect, it } from "vitest";

import { canvasNodeInteraction } from "../canvas/konva/scene/canvas-node-interaction";
import type { CanvasScene, CanvasSceneRelation } from "../canvas/konva/scene/canvas-scene";
import {
  createCanvasScenePreviewIndex,
  projectCanvasScenePreview,
} from "../canvas/konva/scene/scene-preview";
import { createSpaceTextPayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

describe("Konva scene preview", () => {
  it("projects only changed layouts and their adjacent relations", () => {
    const left = sceneNode("left", 0);
    const right = sceneNode("right", 320);
    const unrelated = sceneNode("unrelated", 640);
    const adjacent = relation("adjacent", left.node.id, right.node.id);
    const detached = relation("detached", right.node.id, unrelated.node.id);
    const scene: CanvasScene = {
      nodes: [left, right, unrelated],
      nodesById: new Map([left, right, unrelated].map((item) => [item.node.id, item])),
      contentNodes: [left, right, unrelated],
      sections: [],
      relations: [adjacent, detached],
    };
    const preview = projectCanvasScenePreview(
      createCanvasScenePreviewIndex(scene),
      new Map([
        [
          "left",
          {
            nodeId: "left",
            frame: { ...left.layout.frame, width: 480 },
            appearance: { typographyScale: 2 },
          },
        ],
      ]),
    );
    expect(preview.layouts.get("left")).toMatchObject({
      frame: { width: 480 },
      appearance: { typographyScale: 2 },
    });
    expect(preview.layouts.has("right")).toBe(false);
    expect(preview.relations.has(adjacent.id)).toBe(true);
    expect(preview.relations.has(detached.id)).toBe(false);
  });
});

function sceneNode(id: string, x: number) {
  const node: SpaceCanvasNode = {
    id,
    type: "text",
    data: createSpaceTextPayload(id),
    initialPosition: { x, y: 0 },
  };
  return {
    node,
    interaction: canvasNodeInteraction(node),
    layout: {
      nodeId: id,
      frame: { x, y: 0, width: 240, height: 40, rotation: 0 },
      sizeAuthority: "frame" as const,
      zIndex: 0,
    },
    paintOrder: 0,
  };
}

function relation(id: string, sourceNodeId: string, targetNodeId: string): CanvasSceneRelation {
  return {
    id,
    key: `relation:${id}`,
    kind: "relation",
    label: "related",
    color: "neutral",
    relation: {
      id,
      spaceId: "space",
      sourceId: sourceNodeId,
      targetId: targetNodeId,
      kind: "related",
      rationale: "",
      status: "active",
      actorKind: "human",
      createdAt: 0,
      updatedAt: 0,
    },
    sourceNodeId,
    targetNodeId,
    parallelOffset: 0,
    bounds: { x: 0, y: 0, width: 320, height: 40 },
    midpoint: { x: 160, y: 20 },
    points: [0, 20, 80, 20, 240, 20, 320, 20],
  };
}
