import { shallowReactive, watch, type Ref } from "vue";

import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";
import type { CanvasScene, CanvasSceneRelation } from "./canvas-scene";
import { relationGeometry } from "./relation-path";

export interface CanvasScenePreview {
  layouts: ReadonlyMap<string, CanvasNodeLayoutV1>;
  relations: ReadonlyMap<string, CanvasSceneRelation>;
}

export interface CanvasScenePreviewIndex {
  layouts: ReadonlyMap<string, CanvasNodeLayoutV1>;
  relationsByNodeId: ReadonlyMap<string, readonly CanvasSceneRelation[]>;
}

export function createCanvasScenePreviewIndex(scene: CanvasScene): CanvasScenePreviewIndex {
  const relationsByNodeId = new Map<string, CanvasSceneRelation[]>();
  for (const relation of scene.relations) {
    appendRelation(relationsByNodeId, relation.sourceNodeId, relation);
    appendRelation(relationsByNodeId, relation.targetNodeId, relation);
  }
  return {
    layouts: new Map(scene.nodes.map(({ layout }) => [layout.nodeId, layout])),
    relationsByNodeId,
  };
}

export function projectCanvasScenePreview(
  index: CanvasScenePreviewIndex,
  updates: ReadonlyMap<string, CanvasGestureUpdate>,
): CanvasScenePreview {
  if (updates.size === 0) return { layouts: new Map(), relations: new Map() };
  const layouts = new Map<string, CanvasNodeLayoutV1>();
  const affectedRelations = new Map<string, CanvasSceneRelation>();
  for (const [nodeId, update] of updates) {
    const layout = index.layouts.get(nodeId);
    if (!layout) continue;
    layouts.set(nodeId, previewLayout(layout, update));
    for (const relation of index.relationsByNodeId.get(nodeId) ?? []) {
      affectedRelations.set(relation.id, relation);
    }
  }
  const relations = new Map<string, CanvasSceneRelation>();
  for (const relation of affectedRelations.values()) {
    const projected = previewRelation(relation, index.layouts, layouts);
    if (projected) relations.set(relation.id, projected);
  }
  return { layouts, relations };
}

export function useCanvasScenePreview(
  index: Readonly<Ref<CanvasScenePreviewIndex>>,
  updates: Readonly<Ref<ReadonlyMap<string, CanvasGestureUpdate>>>,
): CanvasScenePreview {
  const layouts = shallowReactive(new Map<string, CanvasNodeLayoutV1>());
  const relations = shallowReactive(new Map<string, CanvasSceneRelation>());
  watch(
    [index, updates],
    () => {
      const projected = projectCanvasScenePreview(index.value, updates.value);
      synchronizePreviewMap(layouts, projected.layouts);
      synchronizePreviewMap(relations, projected.relations);
    },
    { immediate: true, flush: "sync" },
  );
  return { layouts, relations };
}

function synchronizePreviewMap<K, V>(target: Map<K, V>, source: ReadonlyMap<K, V>) {
  for (const key of target.keys()) if (!source.has(key)) target.delete(key);
  for (const [key, value] of source) target.set(key, value);
}

function appendRelation(
  index: Map<string, CanvasSceneRelation[]>,
  nodeId: string,
  relation: CanvasSceneRelation,
) {
  const relations = index.get(nodeId) ?? [];
  relations.push(relation);
  index.set(nodeId, relations);
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
  canonicalLayouts: ReadonlyMap<string, CanvasNodeLayoutV1>,
  previewLayouts: ReadonlyMap<string, CanvasNodeLayoutV1>,
) {
  const source = previewLayouts.get(relation.sourceNodeId) ?? canonicalLayouts.get(relation.sourceNodeId);
  const target = previewLayouts.get(relation.targetNodeId) ?? canonicalLayouts.get(relation.targetNodeId);
  if (!source || !target) return undefined;
  const geometry = relationGeometry(source.frame, target.frame, relation.parallelOffset);
  return { ...relation, ...geometry };
}
