import { computed } from "vue";

import { isSpaceSemanticNode } from "../../../knowledge/model/semantic-node";
import { useSpaceStore } from "../../../session/space-store";
import type {
  CanvasAlignMode,
  CanvasAutoLayoutDirection,
  CanvasDistributeMode,
  CanvasLayerMove,
} from "../../contracts/canvas-editor-port";
import { getCanvasEditorSession } from "../../editor-session";
import type { CanvasNodeAppearance } from "../../schema/document";
import { openSpaceNode } from "../../selection-actions";
import { useCanvasSelectionProjection } from "../../use-canvas-selection-projection";
import { useContextDock } from "../../use-context-dock";
import { useCanvasEditorProtection } from "../use-canvas-editor-protection";
import { useCanvasNodeExport } from "../use-canvas-node-export";
import { useCanvasOperationRunner } from "../use-canvas-operation-runner";
import { createCanvasStructureSelectionActions } from "./canvas-structure-selection-actions";
import { canvasSelectionProductState } from "./selection-product-state";
import { useCanvasReparentActions } from "./use-canvas-reparent-actions";
import { useCanvasSelectedMedia } from "./use-canvas-selected-media";

export function useCanvasSelectionActions() {
  const store = useSpaceStore();
  const dock = useContextDock();
  const editorSession = getCanvasEditorSession();
  const { allows, run: runNodeOperation } = useCanvasOperationRunner();
  const runSelectionMutation = (operation: () => Promise<unknown>) =>
    runNodeOperation("structure", operation);
  const { selection, selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
  const state = computed(() =>
    canvasSelectionProductState({
      nodes: store.nodes,
      selectedIds: selection.value.nodeIds,
      selectedStructureNodeIds: selection.value.structureNodeIds,
      snapshot: store.editSnapshot,
    }),
  );
  const editorProtection = useCanvasEditorProtection(selection, () => state.value.structureNodeIds);
  const exportSelected = useCanvasNodeExport(state, selectedNodes);
  const visible = computed(
    () =>
      Boolean(selection.value.nodeIds.length) ||
      Boolean(selection.value.structureNodeIds.length) ||
      Boolean(selection.value.relationIds.length),
  );
  const selectedAppearance = computed(() => selection.value.singleNodeAppearance);
  const { selectedImageNode, selectedVideoNode } = useCanvasSelectedMedia(selectedNodes);
  const reparentActions = useCanvasReparentActions({
    state,
    run: runSelectionMutation,
  });
  const structureActions = createCanvasStructureSelectionActions({
    state,
    run: runSelectionMutation,
  });

  function setAppearance(patch: Partial<CanvasNodeAppearance>) {
    if (!allows("edit")) return;
    const nodeId = selection.value.nodeIds.length === 1 ? selection.value.nodeIds[0] : undefined;
    if (!nodeId) return;
    editorSession.activeEditor.value?.setNodeAppearance(nodeId, patch);
  }

  function editVideo() {
    if (!allows("edit")) return;
    const node = selectedVideoNode.value;
    if (node) runNodeOperation("edit", () => store.openVideoEditor(node.id));
  }

  function open() {
    const node = selectedNodes.value[0];
    const effect = node?.data.kind === "note" || node?.data.kind === "markdown" ? "edit" : "inspect";
    runNodeOperation(effect, () => openSpaceNode(node));
  }

  function inspect() {
    dock.openInspect();
  }

  function inspectWorkflow() {
    const ids = selectedNodes.value.map(({ id }) => id);
    if (ids.length) dock.openWorkflowCapture(ids);
  }

  function openWorkflowUse() {
    const ids = selectedNodes.value.map(({ id }) => id);
    if (ids.length) dock.openWorkflowUse(ids);
  }

  function ask() {
    dock.openAsk(state.value.askScope, {
      restart: true,
      nodeIds: state.value.content.map(({ id }) => id),
    });
  }

  function connect() {
    const [source, target] = state.value.content.filter(isSpaceSemanticNode);
    if (source && target)
      runNodeOperation("structure", () => store.createRelation(source.id, target.id, "related"));
  }

  function focus() {
    store.canvasFocusIds = state.value.content.map(({ id }) => id);
  }

  function duplicate() {
    const editor = editorSession.activeEditor.value;
    if (editor) runNodeOperation("clone", () => editor.duplicateSelection());
  }

  function align(mode: CanvasAlignMode) {
    const editor = editorSession.activeEditor.value;
    if (editor) runSelectionMutation(() => editor.alignSelection(mode));
  }

  function distribute(mode: CanvasDistributeMode) {
    const editor = editorSession.activeEditor.value;
    if (editor) runSelectionMutation(() => editor.distributeSelection(mode));
  }

  function autoLayout(direction: CanvasAutoLayoutDirection) {
    const editor = editorSession.activeEditor.value;
    if (editor) runSelectionMutation(() => editor.autoLayoutSelection(direction));
  }

  function moveLayer(mode: CanvasLayerMove) {
    if (!allows("structure")) return;
    editorSession.activeEditor.value?.moveSelectionLayer(mode);
  }

  function toggleLock() {
    runSelectionMutation(() =>
      store.executeSpaceEdit({
        kind: "structure.set-lock",
        rootIds: state.value.structureNodeIds,
        locked: !state.value.allLocked,
      }),
    );
  }

  function moveToSpace(spaceId: string) {
    runSelectionMutation(() => store.moveStructureNodes(state.value.structureNodeIds, spaceId));
  }

  function remove() {
    const editor = editorSession.activeEditor.value;
    if (editor) runSelectionMutation(() => editor.deleteSelection());
  }
  return {
    align,
    autoLayout,
    ask,
    connect,
    createGroup: structureActions.createGroup,
    createSection: structureActions.createSection,
    dissolveGroup: structureActions.dissolveGroup,
    dissolveSection: structureActions.dissolveSection,
    distribute,
    exportSelection: () => runNodeOperation("export", exportSelected),
    duplicate,
    focus,
    fitSection: structureActions.fitSection,
    inspect,
    inspectWorkflow,
    openWorkflowUse,
    editVideo,
    editorProtection,
    moveToSpace,
    moveLayer,
    open,
    parentTargets: reparentActions.targets,
    reparent: reparentActions.reparent,
    remove,
    selectedNodes,
    selectedAppearance,
    selectedImageNode,
    selectedVideoNode,
    setAppearance,
    state,
    toggleLock,
    toggleSectionCollapsed: structureActions.toggleSectionCollapsed,
    visible,
  };
}
