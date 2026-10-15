import type Konva from "konva";
import { computed, shallowRef, useTemplateRef } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { CanvasSurfaceProps } from "../contracts/canvas-surface";
import type { CanvasDocumentV1 } from "../schema/document";
import { CanvasDocumentModel } from "./document/canvas-document-model";
import { useCanvasDocumentSync } from "./document/use-canvas-document-sync";
import { createCanvasContextMenu } from "./interaction/canvas-context-menu";
import { useCanvasInteractionCancellation } from "./interaction/use-canvas-interaction-cancellation";
import { useCanvasManipulations } from "./interaction/use-canvas-manipulations";
import { useConnectorCreation } from "./interaction/use-connector-creation";
import { useNodeContentActivation } from "./interaction/use-node-content-activation";
import { usePointerInteractions } from "./interaction/use-pointer-interactions";
import { useStructureDragAutoPan } from "./interaction/use-structure-drag-auto-pan";
import { useCanvasPalette } from "./rendering/use-canvas-palette";
import { useCanvasScenePreview } from "./scene/use-canvas-scene-preview";
import { useSceneVisibility } from "./scene/use-scene-visibility";
import { useCanvasSceneProjections } from "./use-canvas-scene-projections";
import { useRuntimeEditorRegistration } from "./use-runtime-editor-registration";
import { useRuntimeSelection } from "./use-runtime-selection";
import { useRuntimeViewport } from "./use-runtime-viewport";
import { useSectionDrawCandidateBounds } from "./use-section-draw-candidate-bounds";
import { useStructureCanvasInteractions } from "./use-structure-canvas-interactions";
import { useCameraStage } from "./viewport/use-camera-stage";

export function useKonvaCanvasRuntime(
  props: CanvasSurfaceProps,
  documentChanged: (document: CanvasDocumentV1) => void,
) {
  const host = useTemplateRef<HTMLElement>("host");
  const stageComponent = useTemplateRef<VueKonvaRef<Konva.Stage>>("stage");
  const stage = shallowRef<Konva.Stage>();
  const model = new CanvasDocumentModel(props.document, { documentChanged });
  const layouts = computed(() => model.sceneDocumentState.value.nodes);
  const viewport = useRuntimeViewport(props, host, model);
  useCameraStage(viewport.camera, stage);
  const temporarilyExpandedSectionId = shallowRef<string>();
  const structurePreview = shallowRef<SpaceEditSnapshot>();
  const projections = useCanvasSceneProjections(
    props,
    model.sceneDocumentState,
    temporarilyExpandedSectionId,
    structurePreview,
  );
  const { selection, transformPolicy, canTransformGroups } = useRuntimeSelection(props, projections);
  const structureInteractions = useStructureCanvasInteractions({
    clearSelection: selection.clear,
    events: props.events,
    scene: projections.structureScene,
    setStructureSelection: selection.setStructureNodeIds,
    snapshot: () => props.sceneInput.editSnapshot,
    temporarilyExpandedSectionId,
  });
  const manipulations = useCanvasManipulations({
    props,
    model,
    projections,
    selection,
    stage,
    viewport,
    structureInteractions,
    structurePreview,
    temporarilyExpandedSectionId,
  });
  const { preview, gestures, structureGestures, directManipulation, cancelGesture } = manipulations;
  const scenePreview = useCanvasScenePreview(projections.previewIndex, preview.gestureUpdateByNodeId);
  const { activateNode, capturesCanvasInteraction, createTextNode, interactiveNodeId } =
    useNodeContentActivation({
      activate: props.events.onNodeActivate,
      create: props.events.onCreateTextNode,
      tool: () => props.tool,
      reportError: props.events.onError,
      scene: projections.scene,
      selection,
    });
  const palette = useCanvasPalette();
  const visibility = useSceneVisibility({
    activeNodeId: interactiveNodeId,
    camera: viewport.camera,
    nodeIds: selection.nodeIds,
    sectionIds: selection.structureNodeIds,
    sections: computed(() => projections.structureScene.value.sections),
    relationIds: selection.relationIds,
    scene: projections.scene,
    size: viewport.size,
  });
  const pointer = usePointerInteractions({
    cancelActiveGesture: () => {
      nodeDrag.cancel();
      return cancelGesture();
    },
    host,
    selection,
    viewport,
    tool: computed(() => props.tool),
    reportError: props.events.onError,
    createTextNode,
    createSection: props.events.onCreateSection,
  });
  const contextMenu = createCanvasContextMenu({ host, selection, stage });
  const nodeDrag = useStructureDragAutoPan({
    gestures: structureGestures,
    viewport,
    onBegin: pointer.markSelectionDragged,
    snapshot: () => props.sceneInput.editSnapshot,
  });
  const connectors = useConnectorCreation({
    createRelation: props.events.onCreateRelation,
    reportError: props.events.onError,
    pointer,
    scene: projections.scene,
    viewport,
  });
  const cancellation = useCanvasInteractionCancellation({
    cancelConnector: connectors.cancel,
    cancelGesture,
    cancelNodeDrag: nodeDrag.cancel,
    cancelPointer: pointer.pointerCancel,
    cancelStructurePreview: structureInteractions.clearExpansionPreview,
    clearSelection: selection.clear,
    connectorDraft: connectors.draft,
    directManipulation,
    directManipulationChanged: props.events.onDirectManipulationChange,
    enterableContentId: interactiveNodeId,
    exitGroup: structureInteractions.exitGroup,
    groupScopePath: () => props.groupScopePath,
    interactionEnabled: () => props.interactionEnabled,
    marquee: selection.marqueeRect,
    panning: viewport.panning,
    tool: () => props.tool,
  });
  const sectionCandidateBounds = useSectionDrawCandidateBounds(
    pointer.sectionDraft,
    () => props.sceneInput.editSnapshot,
    projections.structureBoundsById,
  );
  useCanvasDocumentSync(() => props.document, model, viewport.setViewport, viewport.getViewport);
  useRuntimeEditorRegistration({
    ...cancellation,
    gestures,
    nudgeSelection: structureGestures.nudge,
    host,
    layouts,
    model,
    palette,
    pointer,
    projections,
    props,
    scene: projections.scene,
    selection,
    stage,
    stageComponent,
    viewport,
  });
  return {
    activateNode,
    capturesCanvasInteraction,
    connectors,
    contextMenu,
    finishDrag: () => void nodeDrag.end().catch(props.events.onError),
    finishTransform: () => void gestures.finishTransform().catch(props.events.onError),
    ...manipulations,
    interactiveNodeId,
    nodeDrag,
    palette,
    pointer,
    scene: projections.scene,
    structure: projections,
    structureInteractions,
    scenePreview,
    sectionCandidateBounds,
    selection,
    stage,
    transformPolicy,
    canTransformGroups,
    viewport,
    visibility,
  };
}
