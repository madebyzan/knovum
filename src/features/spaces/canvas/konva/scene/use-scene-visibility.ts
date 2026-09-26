import { useThrottleFn } from "@vueuse/core";
import { computed, shallowRef, watch, type Ref } from "vue";

import type { CanvasScene } from "./scene-projection";
import { visibleSceneNodeIds, visibleSceneRelationIds } from "./viewport-culling";

interface SceneVisibilityOptions {
  nodeIds: Readonly<Ref<ReadonlySet<string>>>;
  relationIds: Readonly<Ref<ReadonlySet<string>>>;
  scene: Readonly<Ref<CanvasScene>>;
  size: { width: number; height: number };
  viewport: Readonly<Ref<{ x: number; y: number; zoom: number }>>;
}

export function useSceneVisibility(options: SceneVisibilityOptions) {
  const culledNodeIds = shallowRef<ReadonlySet<string>>(new Set());
  const culledRelationIds = shallowRef<ReadonlySet<string>>(new Set());
  const selectedRelationIds = computed(
    () => new Set([...options.relationIds.value].map((id) => `relation:${id}`)),
  );
  const visibleNodeIds = computed(() => {
    const ids = new Set(culledNodeIds.value);
    for (const id of options.nodeIds.value) ids.add(id);
    return ids;
  });
  const visibleRelationIds = computed(() => {
    const ids = new Set(culledRelationIds.value);
    for (const id of selectedRelationIds.value) ids.add(id);
    return ids;
  });

  function updateVisibility() {
    culledNodeIds.value = visibleSceneNodeIds(
      options.scene.value.nodes,
      options.viewport.value,
      options.size,
    );
    culledRelationIds.value = visibleSceneRelationIds(
      options.scene.value.relations,
      options.viewport.value,
      options.size,
    );
  }

  const scheduleVisibilityUpdate = useThrottleFn(updateVisibility, 80, true, true);
  updateVisibility();
  watch(
    [options.scene, options.viewport, () => options.size.width, () => options.size.height],
    scheduleVisibilityUpdate,
  );
  return { selectedRelationIds, visibleNodeIds, visibleRelationIds };
}
