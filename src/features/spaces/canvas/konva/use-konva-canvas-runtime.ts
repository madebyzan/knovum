import type Konva from "konva";
import { computed, shallowRef, useTemplateRef } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { CanvasDocumentV1 } from "../contract/canvas-document";
import type { CanvasSurfaceProps } from "../contract/canvas-surface";
import { CanvasDocumentModel } from "./document/canvas-document-model";
import { useCanvasDocumentSync } from "./document/use-canvas-document-sync";
import { createCanvasGestureSession } from "./interaction/canvas-gesture-session";
import { useCanvasInteractionCancellation } from "./interaction/use-canvas-interaction-cancellation";
import { useConnectorCreation } from "./interaction/use-connector-creation";
import { useLayoutPreview } from "./interaction/use-layout-preview";
import { useNodeContentActivation } from "./interaction/use-node-content-activation";
import { useNodeDragAutoPan } from "./interaction/use-node-drag-auto-pan";
import { usePointerInteractions } from "./interaction/use-pointer-interactions";
import { useCanvasPalette } from "./presentation/use-canvas-palette";
import { createCanvasScenePreviewIndex, useCanvasScenePreview } from "./scene/scene-preview";
import { useSceneVisibility } from "./scene/use-scene-visibility";
import { projectSelectionTransformPolicy } from "./selection/selection-transform-policy";
import { useCanvasSelection } from "./selection/use-canvas-selection";
import { useCanvasSceneProjections } from "./use-canvas-scene-projections";
import { useRuntimeEditorRegistration } from "./use-runtime-editor-registration";
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
  const sceneDocument = model.sceneDocumentRef;
  const layouts = computed(() => sceneDocument.value.nodes);
  const viewport = useRuntimeViewport(props, host, model);
  const preview = useLayoutPreview();
  const temporarilyExpandedSectionId = shallowRef<string>();
  const projections = useCanvasSceneProjections(props, sceneDocument, temporarilyExpandedSectionId);
  const canonicalScene = projections.scene;
  const lockedNodeIds = projections.lockedNodeIds;
  const previewIndex = computed(() => createCanvasScenePreviewIndex(canonicalScene.value));
  const scenePreview = useCanvasScenePreview(previewIndex, preview.layouts);
  const relationBounds = projections.relationBounds;
  const selection = useCanvasSelection({
    availableStructureNodeIds: computed(() => new Set(props.editSnapshot.structure.map(({ id }) => id))),
    layouts: computed(() => canonicalScene.value.nodes.map(({ layout }) => layout)),
    relationBounds,
    structureBounds: projections.structureBounds,
    changed: props.events.onSelectionChange,
    structureSelectionForNode: projections.structureSelectionForNode,
  });
  const structureInteractions = useStructureCanvasInteractions({
    clearSelection: selection.clear,
    events: props.events,
    scene: projections.structureScene,
    selectedIds: selection.structureNodeIds,
    setStructureSelection: selection.setStructureNodeIds,
    snapshot: () => props.editSnapshot,
    temporarilyExpandedSectionId,
  });
  const palette = useCanvasPalette();
  const visibility = useSceneVisibility({
    nodeIds: selection.nodeIds,
    relationIds: selection.relationIds,
    scene: canonicalScene,
    size: viewport.size,
    viewport: viewport.viewport,
  });
  const transformPolicy = computed(() =>
    projectSelectionTransformPolicy(canonicalScene.value.nodes, selection.nodeIds.value),
  );
  const previewFrames = structureInteractions.withContentDropPreview(preview.update);
  const gestures = createCanvasGestureSession({
    document: model,
    events: props.events,
    layouts: () => layouts.value,
    lockedNodeIds: () => lockedNodeIds.value,
    nodes: () => props.nodes,
    previewFrames,
    selectedNodeIds: () => selection.nodeIds.value,
    snapshot: () => props.editSnapshot,
    temporarilyExpandedSectionId: () => temporarilyExpandedSectionId.value,
    stage: () => stage.value,
    zoom: () => viewport.viewport.value.zoom,
  });
  const { activateNode, capturesCanvasInteraction, createTextNode, interactiveNodeId } =
    useNodeContentActivation({
      activate: props.events.onNodeActivate,
      create: props.events.onCreateTextNode,
      tool: () => props.tool,
      reportError: props.events.onError,
      scene: canonicalScene,
      selection,
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
  const nodeDrag = useNodeDragAutoPan({ gestures, viewport });
  const connectors = useConnectorCreation({
    createRelation: props.events.onCreateRelation,
    reportError: props.events.onError,
    pointer,
    scene: canonicalScene,
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
    () => props.editSnapshot,
    projections.structureBounds,
  );
  const report = (operation: Promise<unknown>) => void operation.catch(props.events.onError);
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
    scene: canonicalScene,
    selection,
    stage,
    stageComponent,
    viewport,
  });
  return {
    activateNode,
    capturesCanvasInteraction,
    connectors,
    finishDrag: (id: string) => report(nodeDrag.end(id)),
    finishTransform: () => report(gestures.finish({ kind: "transform" })),
    gestures,
    host,
    interactiveNodeId,
    nodeDrag,
    palette,
    pointer,
    preview,
    scene: canonicalScene,
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
