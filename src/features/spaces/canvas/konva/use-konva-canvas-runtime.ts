import type Konva from "konva";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { CanvasSurfaceProps } from "../contract/canvas-surface";
import { projectEditorLayouts } from "../document/editor-layout-projection";
import { CanvasDocumentModel } from "./document/canvas-document-model";
import { useCanvasDocumentSync } from "./document/use-canvas-document-sync";
import { createCanvasGestureSession } from "./interaction/canvas-gesture-session";
import { useConnectorCreation } from "./interaction/use-connector-creation";
import { useExternalContentDrop } from "./interaction/use-external-content-drop";
import { useLayoutPreview } from "./interaction/use-layout-preview";
import { useNodeContentActivation } from "./interaction/use-node-content-activation";
import { useNodeDragAutoPan } from "./interaction/use-node-drag-auto-pan";
import { usePointerInteractions } from "./interaction/use-pointer-interactions";
import { useSectionDrawing } from "./interaction/use-section-drawing";
import { useCanvasPalette } from "./presentation/use-canvas-palette";
import { createCanvasScenePreviewIndex, useCanvasScenePreview } from "./scene/scene-preview";
import { projectCanvasScene } from "./scene/scene-projection";
import { useSceneVisibility } from "./scene/use-scene-visibility";
import { projectSelectionTransformPolicy } from "./selection/selection-transform-policy";
import { useCanvasSelection } from "./selection/use-canvas-selection";
import { useStructureScope } from "./selection/use-structure-scope";
import { useEditorRegistration } from "./use-editor-registration";
import { canvasPastePoint } from "./viewport/canvas-paste-point";
import { useStageViewport } from "./viewport/use-stage-viewport";

export function useKonvaCanvasRuntime(props: CanvasSurfaceProps) {
  const host = useTemplateRef<HTMLElement>("host");
  const stageComponent = useTemplateRef<VueKonvaRef<Konva.Stage>>("stage");
  const stage = shallowRef<Konva.Stage>();
  const model = new CanvasDocumentModel(props.document, {
    executeCommand: async (request) => {
      await props.events.onPrepareStructureEdit();
      return props.events.onExecuteSpaceEdit(request);
    },
    replayCommand: props.events.onReplaySpaceEdit,
    reportError: props.events.onError,
    historyEntry: props.events.onHistoryEntry,
    viewportChanged: props.events.onViewportChange,
  });
  const scope = useStructureScope(
    () => model.documentRef.value,
    (ids) => selection.setNodeIds(ids),
  );
  const sceneDocument = model.sceneDocumentRef;
  const layouts = computed(() => projectEditorLayouts(sceneDocument.value));
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
    projectCanvasScene(sceneDocument.value, props.nodes, props.relations, scope.groupId.value),
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
    document: () => model.document,
    scopeGroupId: () => scope.groupId.value,
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
    nodes: () => canonicalScene.value.nodes.map((item) => item.node),
    previewFrames: preview.update,
    selectedNodeIds: () => selection.nodeIds.value,
    setSelection: selection.setNodeIds,
    stage: () => stage.value,
    zoom: () => viewport.viewport.value.zoom,
  });
  const {
    activateNode: activateContent,
    capturesCanvasInteraction,
    createTextNode,
    interactiveNodeId,
  } = useNodeContentActivation({
    activate: props.events.onNodeActivate,
    create: props.events.onCreateTextNode,
    tool: () => props.tool,
    reportError: props.events.onError,
    scene: canonicalScene,
    selection,
  });
  function activateNode(id: string) {
    const target = scope.resolveHit(id);
    const item = model.document.structure.items.find((candidate) => candidate.id === target);
    if (item?.kind === "group") scope.enter(target);
    else if (item?.kind === "section") scope.renameId.value = target;
    else activateContent(target);
  }
  const drawing = useSectionDrawing({
    document: () => model.document,
    zoom: () => viewport.viewport.value.zoom,
    exitScope: () => {
      scope.groupId.value = null;
      selection.clear();
    },
    execute: gestures.runStructureCommand,
    complete: props.events.onStructureToolFinished,
    reportError: props.events.onError,
  });
  watch(
    () => props.tool,
    () => drawing.cancel(),
  );
  const pointer = usePointerInteractions({
    sectionDrawing: drawing,
    resolveHit: scope.resolveHit,
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
    if (!gestures.cancelActiveGesture()) {
      if (drawing.bounds.value) drawing.cancel();
      else if (scope.renameId.value) scope.renameId.value = undefined;
      else if (scope.groupId.value) scope.exit();
      else selection.clear();
    }
  }
  const report = (operation: Promise<unknown>) => void operation.catch(props.events.onError);

  useCanvasDocumentSync(() => props.document, model, viewport.setViewport, viewport.getViewport);
  const editor = useEditorRegistration({
    executeStructureCommand: gestures.runStructureCommand,
    enterGroup: scope.enter,
    exitGroup: scope.exit,
    scopeGroupId: () => scope.groupId.value,
    setEditingScope: scope.setForSelection,
    requestContainerRename: (id) => {
      scope.renameId.value = id;
    },
    cancelInteraction,
    events: props.events,
    focusNodeIds: () => props.focusNodeIds,
    host,
    layouts,
    model,
    nodeById: (id) => props.nodes.find((node) => node.id === id),
    runFrameMutation: gestures.runFrameMutation,
    nudgeSelection: gestures.nudge,
    nodes: () => props.nodes,
    palette: () => palette.value,
    relations: () => props.relations,
    relationBounds: () => relationBounds.value,
    pastePoint: () => canvasPastePoint(pointer.lastCanvasPoint.value, viewport.viewport.value, viewport.size),
    selection,
    selectableNodeIds: () =>
      model.document.structure.items
        .filter((item) => item.parentId === scope.groupId.value)
        .map((item) => item.id),
    stage,
    stageComponent,
    viewport,
  });

  return {
    drawing,
    editor,
    scope,
    model,
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
