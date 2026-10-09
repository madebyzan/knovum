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
  gestureUpdateByNodeId: Readonly<Ref<ReadonlyMap<string, CanvasGestureUpdate>>>,
): CanvasScenePreview {
  const layoutByNodeId = shallowReactive(new Map<string, CanvasNodeLayoutV1>());
  const relationById = shallowReactive(new Map<string, CanvasSceneRelation>());
  watch(
    [index, gestureUpdateByNodeId],
    () => {
      const projected = projectCanvasScenePreview(index.value, gestureUpdateByNodeId.value);
      synchronizePreviewMap(layoutByNodeId, projected.layoutByNodeId);
      synchronizePreviewMap(relationById, projected.relationById);
    },
    { immediate: true, flush: "sync" },
  );
  return { layoutByNodeId, relationById };
}

function synchronizePreviewMap<K, V>(target: Map<K, V>, source: ReadonlyMap<K, V>) {
  for (const key of target.keys()) if (!source.has(key)) target.delete(key);
  for (const [key, value] of source) target.set(key, value);
}
