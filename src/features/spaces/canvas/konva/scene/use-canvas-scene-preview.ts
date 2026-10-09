import { shallowReactive, watch, type Ref } from "vue";

import type { CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";
import type { CanvasSceneRelation } from "./canvas-scene";
import {
  projectCanvasScenePreview,
  type CanvasScenePreview,
  type CanvasScenePreviewIndex,
} from "./scene-preview";

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
