import { useThrottleFn } from "@vueuse/core";
import { computed, shallowRef, watch, type Ref } from "vue";

import type { CanvasScene } from "./canvas-scene";
import { createSceneVisibilityIndex, visibleSceneIds } from "./viewport-culling";

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

  const visibilityIndex = computed(() =>
    createSceneVisibilityIndex(options.scene.value.nodes, options.scene.value.relations),
  );

  function updateVisibility() {
    const visible = visibleSceneIds(visibilityIndex.value, options.viewport.value, options.size);
    culledNodeIds.value = visible.nodes;
    culledRelationIds.value = visible.relations;
  }

  const scheduleVisibilityUpdate = useThrottleFn(updateVisibility, 80, true, true);
  updateVisibility();
  watch(
    [options.scene, options.viewport, () => options.size.width, () => options.size.height],
    scheduleVisibilityUpdate,
  );
  return { selectedRelationIds, visibleNodeIds, visibleRelationIds };
}
