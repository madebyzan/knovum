import { useThrottleFn } from "@vueuse/core";
import { computed, onScopeDispose, shallowRef, watch, type Ref } from "vue";

import { SECTION_HEADER_HEIGHT } from "../../../model/structure/section-metrics";
import type { ProjectedSection } from "../../document/structure-scene";
import type { CanvasCamera } from "../viewport/canvas-camera";
import type { CanvasScene } from "./canvas-scene";
import { createSceneVisibilityIndex, visibleSceneIds } from "./viewport-culling";

/**
 * Culling cadence. The index query is cheap, but each result change mounts or discards node content, so the
 * camera is sampled on a fixed interval instead of every frame; the overscan margin covers the travel in
 * between.
 */
const SAMPLE_INTERVAL_MS = 80;

interface SceneVisibilityOptions {
  activeNodeId: Readonly<Ref<string | undefined>>;
  camera: CanvasCamera;
  nodeIds: Readonly<Ref<ReadonlySet<string>>>;
  relationIds: Readonly<Ref<ReadonlySet<string>>>;
  scene: Readonly<Ref<CanvasScene>>;
  sections: Readonly<Ref<readonly ProjectedSection[]>>;
  sectionIds: Readonly<Ref<ReadonlySet<string>>>;
  size: { width: number; height: number };
}

export function useSceneVisibility(options: SceneVisibilityOptions) {
  const culledSectionIds = shallowRef<ReadonlySet<string>>(new Set());
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
  const visibleContentNodeIds = computed(() => {
    const ids = new Set(culledNodeIds.value);
    if (options.activeNodeId.value) ids.add(options.activeNodeId.value);
    return ids;
  });
  const visibleRelationIds = computed(() => {
    const ids = new Set(culledRelationIds.value);
    for (const id of selectedRelationIds.value) ids.add(id);
    return ids;
  });

  const visibilityIndex = computed(() =>
    createSceneVisibilityIndex(
      options.scene.value.nodes,
      options.scene.value.relations,
      options.sections.value.map(({ section, bounds, layout, temporarilyExpanded }) => ({
        key: section.id,
        bounds: {
          x: bounds.left,
          y: bounds.top,
          width: bounds.right - bounds.left,
          height:
            layout.collapsed && !temporarilyExpanded ? SECTION_HEADER_HEIGHT : bounds.bottom - bounds.top,
        },
      })),
    ),
  );

  function updateVisibility() {
    const visible = visibleSceneIds(visibilityIndex.value, options.camera.get(), options.size);
    culledNodeIds.value = visible.nodes;
    culledSectionIds.value = visible.sections;
    culledRelationIds.value = visible.relations;
  }

  const scheduleVisibilityUpdate = useThrottleFn(updateVisibility, SAMPLE_INTERVAL_MS, true, true);
  updateVisibility();
  watch(
    [options.scene, options.sections, () => options.size.width, () => options.size.height],
    scheduleVisibilityUpdate,
  );
  onScopeDispose(options.camera.observeFrames(scheduleVisibilityUpdate));
  const visibleSections = computed(() =>
    options.sections.value.filter(
      ({ section, temporarilyExpanded }) =>
        culledSectionIds.value.has(section.id) ||
        options.sectionIds.value.has(section.id) ||
        temporarilyExpanded,
    ),
  );
  return { selectedRelationIds, visibleContentNodeIds, visibleNodeIds, visibleRelationIds, visibleSections };
}
