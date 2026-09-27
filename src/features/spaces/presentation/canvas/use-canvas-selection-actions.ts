import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type {
  CanvasAlignMode,
  CanvasAutoLayoutDirection,
  CanvasDistributeMode,
  CanvasLayerMove,
} from "../../canvas/contracts/canvas-editor-port";
import { useCanvasEditorSession, useCanvasSelectionProjection } from "../../canvas/editor-session";
import type { CanvasNodeAppearance } from "../../canvas/schema/document";
import { openSpaceNode } from "../../canvas/selection-actions";
import { useContextDock } from "../../canvas/use-context-dock";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import { useSpaceStore } from "../../session/space-store";
import { useCanvasNodeDownload } from "./canvas-node-download";
import { canvasSelectionProductState } from "./selection-product-state";
import { useCanvasOperationRunner } from "./use-canvas-operation-runner";
import { useCanvasReparentActions } from "./use-canvas-reparent-actions";
import { useCanvasSelectedMedia } from "./use-canvas-selected-media";
import { useCanvasStructureSelectionActions } from "./use-canvas-structure-selection-actions";

export function useCanvasSelectionActions() {
  const store = useSpaceStore();
  const dock = useContextDock();
  const editorSession = useCanvasEditorSession();
  const nodeOperations = useCanvasOperationRunner();
  const runNodeOperation = nodeOperations.run;
  const runSelectionMutation = (operation: () => Promise<unknown>) =>
    runNodeOperation("structure", operation);
  const { selection, selectedNodes } = useCanvasSelectionProjection(() => store.nodes);
  const state = computed(() =>
    canvasSelectionProductState(
      store.nodes,
      selection.value.nodeIds,
      selection.value.structureNodeIds,
      store.editSnapshot,
    ),
  );
  const { t } = useI18n();
  const downloadSelected = useCanvasNodeDownload(store, state, selectedNodes);
  const visible = computed(
    () =>
      Boolean(selection.value.nodeIds.length) ||
      Boolean(selection.value.structureNodeIds.length) ||
      Boolean(selection.value.relationIds.length),
  );
  const selectedAppearance = computed(() => selection.value.singleNodeAppearance);
  const { selectedImageNode, selectedVideoNode } = useCanvasSelectedMedia(selectedNodes);
  const reparentActions = useCanvasReparentActions(
    store,
    state,
    t("spaces.canvas.spaceRoot"),
    runSelectionMutation,
  );
  const structureActions = useCanvasStructureSelectionActions(
    store,
    state,
    { group: t("spaces.canvas.groups.untitled"), section: t("spaces.canvas.sections.untitled") },
    runSelectionMutation,
  );

  function setAppearance(patch: Partial<CanvasNodeAppearance>) {
    if (!nodeOperations.allows("edit")) return;
    const nodeId = selection.value.nodeIds.length === 1 ? selection.value.nodeIds[0] : undefined;
    if (!nodeId) return;
    editorSession.activeEditor.value?.setNodeAppearance(nodeId, patch);
  }

  function editVideo() {
    if (!nodeOperations.allows("edit")) return;
    const node = selectedVideoNode.value;
    if (node) store.openVideoEditor(node.id);
  }

  function open() {
    const node = selectedNodes.value[0];
    const effect = node?.data.kind === "note" || node?.data.kind === "markdown" ? "edit" : "inspect";
    runNodeOperation(effect, () => openSpaceNode(store, node));
  }

  function inspect() {
    dock.openInspect();
  }

  function inspectWorkflow() {
    const ids = selectedNodes.value.map(({ id }) => id);
    if (ids.length) dock.openWorkflowCapture(ids);
  }

  function useWorkflow() {
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
    if (!nodeOperations.allows("structure")) return;
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
    download: () => runNodeOperation("export", downloadSelected),
    duplicate,
    focus,
    fitSection: structureActions.fitSection,
    inspect,
    inspectWorkflow,
    useWorkflow,
    editVideo,
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
