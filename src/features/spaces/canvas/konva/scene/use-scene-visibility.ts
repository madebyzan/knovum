import { useThrottleFn } from "@vueuse/core";
import { computed, onScopeDispose, shallowRef, watch, type Ref } from "vue";

import { SECTION_HEADER_HEIGHT } from "../../../model/structure/section-metrics";
import type { ProjectedSection } from "../../document/structure-scene";
import type { CanvasCamera } from "../viewport/canvas-camera";
import type { CanvasScene } from "./canvas-scene";
import { useContentAdmission } from "./use-content-admission";
import { createSceneVisibilityIndex, sceneOverscan, visibleSceneIds } from "./viewport-culling";

/**
 * Culling cadence. The index query is cheap, but every result change reprojects the whole visible list and
 * patches the keyed scene layers, a cost that does not shrink with the size of the change - sampling every
 * frame would pay it five times as often for the same net work. The camera is therefore sampled on a fixed
 * interval and the travel-aware overscan covers the gap, while content mounting is spread across frames by
 * `useContentAdmission`.
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
  /** The culled extent: the stage surface, so a moving host edge never changes what is mounted. */
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
    const ids = new Set(admittedContentIds.value);
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

  let sampled = options.camera.get();

  function updateVisibility() {
    const camera = options.camera.get();
    const travel = { x: camera.x - sampled.x, y: camera.y - sampled.y };
    sampled = camera;
    const visible = visibleSceneIds(visibilityIndex.value, camera, options.size, sceneOverscan(travel));
    culledNodeIds.value = visible.nodes;
    culledSectionIds.value = visible.sections;
    culledRelationIds.value = visible.relations;
  }

  updateVisibility();
  const admittedContentIds = useContentAdmission(options.camera, culledNodeIds);
  const scheduleVisibilityUpdate = useThrottleFn(updateVisibility, SAMPLE_INTERVAL_MS, true, true);
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
