import type Konva from "konva";
import { computed, shallowRef, useTemplateRef } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { CanvasSurfaceProps } from "../contracts/canvas-surface";
import type { CanvasDocumentV1 } from "../schema/document";
import { CanvasDocumentModel } from "./document/canvas-document-model";
import { useCanvasDocumentSync } from "./document/use-canvas-document-sync";
import { createCanvasGestureSession } from "./interaction/canvas-gesture-session";
import { useCanvasContextMenu } from "./interaction/use-canvas-context-menu";
import { useCanvasInteractionCancellation } from "./interaction/use-canvas-interaction-cancellation";
import { useConnectorCreation } from "./interaction/use-connector-creation";
import { useLayoutPreview } from "./interaction/use-layout-preview";
import { useNodeContentActivation } from "./interaction/use-node-content-activation";
import { useNodeDragAutoPan } from "./interaction/use-node-drag-auto-pan";
import { usePointerInteractions } from "./interaction/use-pointer-interactions";
import { useCanvasPalette } from "./presentation/use-canvas-palette";
import { useCanvasScenePreview } from "./scene/use-canvas-scene-preview";
import { useSceneVisibility } from "./scene/use-scene-visibility";
import { useCanvasSceneProjections } from "./use-canvas-scene-projections";
import { useRuntimeEditorRegistration } from "./use-runtime-editor-registration";
import { useRuntimeSelection } from "./use-runtime-selection";
import { useRuntimeViewport } from "./use-runtime-viewport";
import { useSectionDrawCandidateBounds } from "./use-section-draw-candidates";
import { useStructureCanvasInteractions } from "./use-structure-canvas-interactions";

export function useKonvaCanvasRuntime(
  props: CanvasSurfaceProps,
  documentChanged: (document: CanvasDocumentV1) => void,
) {
  const host = useTemplateRef<HTMLElement>("host");
  const stageComponent = useTemplateRef<VueKonvaRef<Konva.Stage>>("stage");
  const stage = shallowRef<Konva.Stage>();
  const model = new CanvasDocumentModel(props.document, {
    documentChanged,
    viewportChanged: props.events.onViewportChange,
  });
  const sceneDocument = model.sceneDocumentState;
  const layouts = computed(() => sceneDocument.value.nodes);
  const viewport = useRuntimeViewport(props, host, model);
  const temporarilyExpandedSectionId = shallowRef<string>();
  const projections = useCanvasSceneProjections(props, sceneDocument, temporarilyExpandedSectionId);
  const { selection, transformPolicy } = useRuntimeSelection(props, projections);
  const structureInteractions = useStructureCanvasInteractions({
    clearSelection: selection.clear,
    events: props.events,
    scene: projections.structureScene,
    selectedIds: selection.structureNodeIds,
    setStructureSelection: selection.setStructureNodeIds,
    snapshot: () => props.sceneInput.editSnapshot,
    temporarilyExpandedSectionId,
  });
  const preview = useLayoutPreview(structureInteractions.previewContentMove);
  const scenePreview = useCanvasScenePreview(projections.previewIndex, preview.layouts);
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
    nodeIds: selection.nodeIds,
    sectionIds: selection.structureNodeIds,
    sections: computed(() => projections.structureScene.value.sections),
    relationIds: selection.relationIds,
    scene: projections.scene,
    size: viewport.size,
    viewport: viewport.viewport,
  });
  const gestures = createCanvasGestureSession({
    document: model,
    events: props.events,
    layouts: () => layouts.value,
    lockedNodeIds: () => projections.lockedNodeIds.value,
    nodes: () => props.sceneInput.nodes,
    previewFrames: preview.update,
    selectedNodeIds: () => selection.nodeIds.value,
    snapshot: () => props.sceneInput.editSnapshot,
    temporarilyExpandedSectionId: () => temporarilyExpandedSectionId.value,
    stage: () => stage.value,
    zoom: () => viewport.viewport.value.zoom,
  });
  const pointer = usePointerInteractions({
    cancelActiveGesture: gestures.cancelActiveGesture,
    host,
    selection,
    viewport,
    tool: computed(() => props.tool),
    reportError: props.events.onError,
    createTextNode,
    createSection: props.events.onCreateSection,
  });
  const contextMenu = useCanvasContextMenu({ host, selection, stage });
  const nodeDrag = useNodeDragAutoPan({ gestures, viewport });
  const connectors = useConnectorCreation({
    createRelation: props.events.onCreateRelation,
    reportError: props.events.onError,
    pointer,
    scene: projections.scene,
    viewport,
  });
  const { cancelInteraction } = useCanvasInteractionCancellation({
    cancelConnector: connectors.cancel,
    cancelGesture: gestures.cancelActiveGesture,
    cancelNodeDrag: nodeDrag.cancel,
    cancelPointer: pointer.pointerCancel,
    cancelStructurePreview: structureInteractions.clearExpansionPreview,
    clearSelection: selection.clear,
    connectorDraft: connectors.draft,
    directManipulation: gestures.directManipulation,
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
    projections.structureBounds,
  );
  useCanvasDocumentSync(() => props.document, model, viewport.setViewport, viewport.getViewport);
  useRuntimeEditorRegistration({
    cancelInteraction,
    gestures,
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
    finishDrag: (id: string) => void nodeDrag.end(id).catch(props.events.onError),
    finishTransform: () => void gestures.finish({ kind: "transform" }).catch(props.events.onError),
    gestures,
    host,
    interactiveNodeId,
    nodeDrag,
    palette,
    pointer,
    preview,
    scene: projections.scene,
    structure: projections,
    structureInteractions,
    scenePreview,
    sectionCandidateBounds,
    selection,
    stage,
    transformPolicy,
    viewport,
    visibility,
  };
}
