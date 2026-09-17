import type Konva from "konva";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { CanvasDocumentV1 } from "../contract/canvas-document";
import type { CanvasSurfaceProps } from "../contract/canvas-surface";
import { CanvasDocumentModel } from "./document/canvas-document-model";
import { useCanvasDocumentSync } from "./document/use-canvas-document-sync";
import { createCanvasGestureSession } from "./interaction/canvas-gesture-session";
import { useConnectorCreation } from "./interaction/use-connector-creation";
import { useExternalContentDrop } from "./interaction/use-external-content-drop";
import { useLayoutPreview } from "./interaction/use-layout-preview";
import { useNodeContentActivation } from "./interaction/use-node-content-activation";
import { useNodeDragAutoPan } from "./interaction/use-node-drag-auto-pan";
import { usePointerInteractions } from "./interaction/use-pointer-interactions";
import { useCanvasPalette } from "./presentation/use-canvas-palette";
import { createCanvasScenePreviewIndex, useCanvasScenePreview } from "./scene/scene-preview";
import { projectCanvasScene } from "./scene/scene-projection";
import { useSceneVisibility } from "./scene/use-scene-visibility";
import { projectSelectionTransformPolicy } from "./selection/selection-transform-policy";
import { useCanvasSelection } from "./selection/use-canvas-selection";
import { useEditorRegistration } from "./use-editor-registration";
import { canvasPastePoint } from "./viewport/canvas-paste-point";
import { useStageViewport } from "./viewport/use-stage-viewport";

export function useKonvaCanvasRuntime(
  props: CanvasSurfaceProps,
  documentChanged: (document: CanvasDocumentV1) => void,
) {
  const host = useTemplateRef<HTMLElement>("host");
  const stageComponent = useTemplateRef<VueKonvaRef<Konva.Stage>>("stage");
  const stage = shallowRef<Konva.Stage>();
  const model = new CanvasDocumentModel(props.document, {
    documentChanged,
    historyEntry: props.events.onHistoryEntry,
    viewportChanged: props.events.onViewportChange,
  });
  const sceneDocument = model.sceneDocumentRef;
  const layouts = computed(() => sceneDocument.value.nodes);
  const viewport = useStageViewport({
    host,
    initialViewport: model.document.viewport,
    update: (next) => model.setViewport(next),
  });
  useExternalContentDrop({
    host,
    viewport: () => viewport.viewport.value,
    insertText: props.events.onInsertText,
    insertUrl: props.events.onInsertUrl,
    reportError: props.events.onError,
  });
  const preview = useLayoutPreview();
  const canonicalScene = computed(() =>
    projectCanvasScene(sceneDocument.value, props.nodes, props.relations),
  );
  const previewIndex = computed(() => createCanvasScenePreviewIndex(canonicalScene.value));
  const scenePreview = useCanvasScenePreview(previewIndex, preview.layouts);
  const relationBounds = computed(
    () =>
      new Map(
        canonicalScene.value.relations.map(({ id, midpoint }) => [
          id,
          { x: midpoint.x, y: midpoint.y, width: 0, height: 0 },
        ]),
      ),
  );
  const selection = useCanvasSelection({
    layouts: computed(() => canonicalScene.value.nodes.map(({ layout }) => layout)),
    relationBounds,
    changed: props.events.onSelectionChange,
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
  const gestures = createCanvasGestureSession({
    document: model,
    events: props.events,
    layouts: () => layouts.value,
    nodes: () => props.nodes,
    palette: () => palette.value,
    previewFrames: preview.update,
    sectionNodeIds: () =>
      new Set(props.nodes.filter(({ data }) => data.kind === "section").map(({ id }) => id)),
    selectedNodeIds: () => selection.nodeIds.value,
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
  });
  const nodeDrag = useNodeDragAutoPan({ gestures, viewport });
  watch(() => gestures.directManipulation.value !== undefined, props.events.onDirectManipulationChange);
  const connectors = useConnectorCreation({
    createRelation: props.events.onCreateRelation,
    reportError: props.events.onError,
    pointer,
    scene: canonicalScene,
    viewport,
  });
  watch(
    () => [props.tool, props.interactionEnabled] as const,
    () => {
      pointer.pointerCancel();
      connectors.cancel();
      interactiveNodeId.value = undefined;
    },
  );

  function cancelInteraction() {
    if (interactiveNodeId.value) {
      interactiveNodeId.value = undefined;
      return;
    }
    if (connectors.draft.value) return connectors.cancel();
    if (selection.marqueeRect.value || viewport.panning.value) return pointer.pointerCancel();
    nodeDrag.cancel();
    if (!gestures.cancelActiveGesture()) selection.clear();
  }
  const report = (operation: Promise<unknown>) => void operation.catch(props.events.onError);

  useCanvasDocumentSync(() => props.document, model, viewport.setViewport, viewport.getViewport);
  useEditorRegistration({
    cancelInteraction,
    events: props.events,
    focusNodeIds: () => props.focusNodeIds,
    host,
    layouts,
    model,
    nodeById: (id) => canonicalScene.value.nodesById.get(id)?.node,
    runFrameMutation: gestures.runFrameMutation,
    nudgeSelection: gestures.nudge,
    nodes: () => props.nodes,
    palette: () => palette.value,
    relations: () => props.relations,
    relationBounds: () => relationBounds.value,
    pastePoint: () => canvasPastePoint(pointer.lastCanvasPoint.value, viewport.viewport.value, viewport.size),
    selection,
    selectableNodeIds: () => canonicalScene.value.nodes.map(({ node }) => node.id),
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
    scenePreview,
    selection,
    stage,
    transformPolicy,
    viewport,
    visibility,
  };
}
