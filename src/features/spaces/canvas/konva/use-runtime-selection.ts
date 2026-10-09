import { computed } from "vue";

import type { CanvasSurfaceProps } from "../contracts/canvas-surface";
import { projectSelectionTransformPolicy } from "./selection/selection-transform-policy";
import { useCanvasSelection } from "./selection/use-canvas-selection";
import type { useCanvasSceneProjections } from "./use-canvas-scene-projections";

export function useRuntimeSelection(
  props: CanvasSurfaceProps,
  projections: ReturnType<typeof useCanvasSceneProjections>,
) {
  const selection = useCanvasSelection({
    availableStructureNodeIds: computed(
      () => new Set(props.sceneInput.editSnapshot.structure.map(({ id }) => id)),
    ),
    layouts: computed(() => projections.scene.value.nodes.map(({ layout }) => layout)),
    relationBoundsById: projections.relationBoundsById,
    structureBoundsById: projections.structureBoundsById,
    changed: props.events.onSelectionChange,
    structureSelectionForNode: projections.structureSelectionForNode,
  });
  const transformPolicy = computed(() =>
    projectSelectionTransformPolicy(projections.scene.value.nodes, selection.nodeIds.value),
  );
  return { selection, transformPolicy };
}
