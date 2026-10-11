import type Konva from "konva";
import { computed, type Ref } from "vue";

import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import { createStructureIndex } from "../../../model/structure/structure-index";
import type { CanvasSurfaceProps } from "../../contracts/canvas-surface";
import { currentSelectionRoots } from "../../selection/selection-roots";
import type { CanvasDocumentModel } from "../document/canvas-document-model";
import type { useCanvasSelection } from "../selection/use-canvas-selection";
import type { useCanvasSceneProjections } from "../use-canvas-scene-projections";
import type { useStructureCanvasInteractions } from "../use-structure-canvas-interactions";
import type { useStageViewport } from "../viewport/use-stage-viewport";
import { createCanvasGestureSession } from "./canvas-gesture-session";
import { useLayoutPreview } from "./use-layout-preview";
import { useStructureGestureSession } from "./use-structure-gesture-session";

interface CanvasManipulationOptions {
  props: CanvasSurfaceProps;
  model: CanvasDocumentModel;
  projections: ReturnType<typeof useCanvasSceneProjections>;
  selection: ReturnType<typeof useCanvasSelection>;
  stage: Ref<Konva.Stage | undefined>;
  viewport: ReturnType<typeof useStageViewport>;
  structureInteractions: ReturnType<typeof useStructureCanvasInteractions>;
  structurePreview: Ref<SpaceEditSnapshot | undefined>;
  temporarilyExpandedSectionId: Ref<string | undefined>;
}

export function useCanvasManipulations(options: CanvasManipulationOptions) {
  const { props, model, selection, viewport } = options;
  const preview = useLayoutPreview();
  const gestures = createCanvasGestureSession({
    document: model,
    events: props.events,
    layouts: () => model.sceneDocumentState.value.nodes,
    nodes: () => props.sceneInput.nodes,
    previewFrames: preview.update,
    selectedNodeIds: () => selection.nodeIds.value,
    snapshot: () => props.sceneInput.editSnapshot,
    stage: () => options.stage.value,
    zoom: () => viewport.viewport.value.zoom,
  });
  const structureGestures = useStructureGestureSession({
    snapshot: () => props.sceneInput.editSnapshot,
    selectedRootIds: () =>
      currentSelectionRoots(
        selection.nodeIds.value,
        selection.structureNodeIds.value,
        createStructureIndex(props.sceneInput.editSnapshot.structure),
      ).map(({ id }) => id),
    bounds: () => options.projections.structureBoundsById.value,
    execute: gestures.executeCommand,
    preview: options.structurePreview,
    previewDrop: options.structureInteractions.previewDrop,
    expandedSectionId: () => options.temporarilyExpandedSectionId.value,
    zoom: () => viewport.viewport.value.zoom,
  });
  return {
    preview,
    gestures,
    structureGestures,
    directManipulation: computed(
      () => structureGestures.directManipulation.value ?? gestures.directManipulation.value,
    ),
    feedback: computed(() => structureGestures.feedback.value ?? gestures.feedback.value),
    cancelGesture: () => structureGestures.cancel() || gestures.cancelActiveGesture(),
  };
}
