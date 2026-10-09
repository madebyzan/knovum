import type { CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";
import type { CanvasScene, CanvasSceneRelation } from "./canvas-scene";
import { relationGeometry } from "./relation-path";

export interface CanvasScenePreview {
  layoutByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>;
  relationById: ReadonlyMap<string, CanvasSceneRelation>;
}

export interface CanvasScenePreviewIndex {
  layoutByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>;
  relationsByNodeId: ReadonlyMap<string, readonly CanvasSceneRelation[]>;
}

export function createCanvasScenePreviewIndex(scene: CanvasScene): CanvasScenePreviewIndex {
  const relationsByNodeId = new Map<string, CanvasSceneRelation[]>();
  for (const relation of scene.relations) {
    appendRelation(relationsByNodeId, relation.sourceNodeId, relation);
    appendRelation(relationsByNodeId, relation.targetNodeId, relation);
  }
  return {
    layoutByNodeId: new Map(scene.nodes.map(({ layout }) => [layout.nodeId, layout])),
    relationsByNodeId,
  };
}

export function projectCanvasScenePreview(
  index: CanvasScenePreviewIndex,
  gestureUpdateByNodeId: ReadonlyMap<string, CanvasGestureUpdate>,
): CanvasScenePreview {
  if (gestureUpdateByNodeId.size === 0) return { layoutByNodeId: new Map(), relationById: new Map() };
  const layoutByNodeId = new Map<string, CanvasNodeLayoutV1>();
  const affectedRelationById = new Map<string, CanvasSceneRelation>();
  for (const [nodeId, update] of gestureUpdateByNodeId) {
    const layout = index.layoutByNodeId.get(nodeId);
    if (!layout) continue;
    layoutByNodeId.set(nodeId, previewLayout(layout, update));
    for (const relation of index.relationsByNodeId.get(nodeId) ?? []) {
      affectedRelationById.set(relation.id, relation);
    }
  }
  const relationById = new Map<string, CanvasSceneRelation>();
  for (const relation of affectedRelationById.values()) {
    const projected = previewRelation(relation, index.layoutByNodeId, layoutByNodeId);
    if (projected) relationById.set(relation.id, projected);
  }
  return { layoutByNodeId, relationById };
}

function appendRelation(
  relationsByNodeId: Map<string, CanvasSceneRelation[]>,
  nodeId: string,
  relation: CanvasSceneRelation,
) {
  const relations = relationsByNodeId.get(nodeId) ?? [];
  relations.push(relation);
  relationsByNodeId.set(nodeId, relations);
}

function previewLayout(layout: CanvasNodeLayoutV1, update: CanvasGestureUpdate): CanvasNodeLayoutV1 {
  return {
    ...layout,
    frame: update.frame,
    appearance: "appearance" in update ? (update.appearance ?? undefined) : layout.appearance,
  };
}

function previewRelation(
  relation: CanvasSceneRelation,
  canonicalLayoutByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>,
  previewLayoutByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>,
) {
  const source =
    previewLayoutByNodeId.get(relation.sourceNodeId)?.frame ??
    canonicalLayoutByNodeId.get(relation.sourceNodeId)?.frame ??
    relation.sourceFrame;
  const target =
    previewLayoutByNodeId.get(relation.targetNodeId)?.frame ??
    canonicalLayoutByNodeId.get(relation.targetNodeId)?.frame ??
    relation.targetFrame;
  const geometry = relationGeometry(source, target, relation.parallelOffset);
  return { ...relation, ...geometry };
}
