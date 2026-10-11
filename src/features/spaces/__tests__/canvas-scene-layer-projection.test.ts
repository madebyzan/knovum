import { describe, expect, it } from "vitest";
import { shallowReactive } from "vue";

import { useCanvasSceneLayerProjection } from "../canvas/konva/rendering/use-canvas-scene-layer-projection";
import { projectCanvasScene } from "../canvas/konva/scene/scene-projection";
import { createSpaceNode } from "../canvas/node";
import { createSpaceTextPayload } from "../document-editing/tiptap-content";
import type { SpaceRelation } from "../model/relations/relation";

function scene() {
  const ids = ["first", "second", "third"];
  const relations: SpaceRelation[] = ["first-second", "second-third"].map((id, index) => ({
    id,
    spaceId: "space",
    sourceId: ids[index]!,
    targetId: ids[index + 1]!,
    kind: "related",
    rationale: "",
    status: "active",
    actorKind: "human",
    createdAt: 0,
    updatedAt: 0,
  }));
  return projectCanvasScene({
    document: {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: ids.map((nodeId, zIndex) => ({
        nodeId,
        zIndex,
        sizingMode: "frame",
        frame: { x: zIndex * 300, y: 0, width: 240, height: 40, rotation: 0 },
      })),
    },
    nodes: ids.map((id) => createSpaceNode(createSpaceTextPayload(id), { x: 0, y: 0 }, id)),
    relations,
  });
}

describe("canvas scene layer projection", () => {
  it("preserves paint order independently of spatial query and selection order", () => {
    const props = shallowReactive({
      connectorDraft: undefined,
      scene: scene(),
      selectedNodeIds: new Set(["third", "first"]),
      visibleNodeIds: new Set(["third", "missing", "first"]),
      visibleRelationIds: new Set(["relation:second-third", "relation:first-second", "missing"]),
    });
    const projection = useCanvasSceneLayerProjection(props);
    expect(projection.visibleContentNodes.value.map(({ node }) => node.id)).toEqual(["first", "third"]);
    expect(projection.visibleRelations.value.map(({ id }) => id)).toEqual(["first-second", "second-third"]);
    expect(projection.selectedFrames.value.map(({ x }) => x)).toEqual([0, 600]);
  });

  it("updates visibility, selection and scene identities without retaining stale items", () => {
    const props = shallowReactive({
      connectorDraft: undefined,
      scene: scene(),
      selectedNodeIds: new Set(["first"]),
      visibleNodeIds: new Set(["first"]),
      visibleRelationIds: new Set(["relation:first-second"]),
    });
    const projection = useCanvasSceneLayerProjection(props);
    expect(projection.connectorNode.value?.node.id).toBe("first");
    expect(projection.visibleRelations.value).toHaveLength(1);
    props.selectedNodeIds = new Set(["second"]);
    props.visibleNodeIds = new Set(["second"]);
    expect(projection.connectorNode.value?.node.id).toBe("second");
    expect(projection.visibleContentNodes.value.map(({ node }) => node.id)).toEqual(["second"]);
    props.scene = { ...scene(), relations: [] };
    expect(projection.visibleRelations.value).toEqual([]);
  });
});
