import { describe, expect, it } from "vitest";

import { canvasNodeInteraction } from "../canvas/canvas-node-interaction";
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
      sectionBackdrops: [],
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
    expect(preview.layoutByNodeId.get("left")).toMatchObject({
      frame: { width: 480 },
      appearance: { typographyScale: 2 },
    });
    expect(preview.layoutByNodeId.has("right")).toBe(false);
    expect(preview.relationById.has(adjacent.id)).toBe(true);
    expect(preview.relationById.has(detached.id)).toBe(false);
  });
});

function sceneNode(nodeId: string, x: number) {
  const node: SpaceCanvasNode = {
    id: nodeId,
    type: "text",
    data: createSpaceTextPayload(nodeId),
    initialPosition: { x, y: 0 },
  };
  return {
    node,
    interaction: canvasNodeInteraction(node),
    layout: {
      nodeId: nodeId,
      frame: { x, y: 0, width: 240, height: 40, rotation: 0 },
      sizingMode: "frame" as const,
      zIndex: 0,
    },
    paintOrder: 0,
  };
}

function relation(relationId: string, sourceNodeId: string, targetNodeId: string): CanvasSceneRelation {
  const sourceFrame = { x: 0, y: 0, width: 240, height: 40, rotation: 0 };
  const targetFrame = { x: 320, y: 0, width: 240, height: 40, rotation: 0 };
  return {
    id: relationId,
    key: `relation:${relationId}`,
    kind: "relation",
    label: "related",
    color: "neutral",
    relation: {
      id: relationId,
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
    sourceFrame,
    targetNodeId,
    targetFrame,
    parallelOffset: 0,
    bounds: { x: 0, y: 0, width: 320, height: 40 },
    midpoint: { x: 160, y: 20 },
    points: [0, 20, 80, 20, 240, 20, 320, 20],
  };
}
