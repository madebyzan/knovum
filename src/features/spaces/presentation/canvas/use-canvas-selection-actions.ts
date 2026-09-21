import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { CanvasNodeAppearance } from "../../canvas/contract/canvas-document";
import type {
  CanvasAlignMode,
  CanvasAutoLayoutDirection,
  CanvasDistributeMode,
  CanvasLayerMove,
} from "../../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession, useCanvasSelectionProjection } from "../../canvas/editor-session";
import { openSpaceNode } from "../../canvas/selection-actions";
import { useContextDock } from "../../canvas/use-context-dock";
import { isSpaceSemanticNode } from "../../knowledge/model/semantic-node";
import { useSpaceStore } from "../../session/space-store";
import { canvasSelectionProductState } from "./selection-product-state";
import { useCanvasReparentActions } from "./use-canvas-reparent-actions";
import { useCanvasStructureSelectionActions } from "./use-canvas-structure-selection-actions";

function runCanvasAction(operation: () => Promise<unknown>) {
  void Promise.resolve()
    .then(operation)
    .catch((error) => toast.error(String(error)));
}

export function useCanvasSelectionActions() {
  const store = useSpaceStore();
  const dock = useContextDock();
  const editorSession = useCanvasEditorSession();
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
  const visible = computed(
    () =>
      selection.value.nodeIds.length > 0 ||
      selection.value.structureNodeIds.length > 0 ||
      selection.value.relationIds.length > 0,
  );
  const selectedAppearance = computed(() => selection.value.singleNodeAppearance);
  const reparentActions = useCanvasReparentActions(
    store,
    state,
    t("spaces.canvas.spaceRoot"),
    runCanvasAction,
  );
  const structureActions = useCanvasStructureSelectionActions(
    store,
    state,
    { group: t("spaces.canvas.untitledGroup"), section: t("spaces.canvas.untitledSection") },
    runCanvasAction,
  );

  function setAppearance(patch: Partial<CanvasNodeAppearance>) {
    const nodeId = selection.value.nodeIds.length === 1 ? selection.value.nodeIds[0] : undefined;
    if (!nodeId) return;
    editorSession.activeEditor.value?.setNodeAppearance(nodeId, patch);
  }

  function open() {
    runCanvasAction(() => openSpaceNode(store, selectedNodes.value[0]));
  }

  function inspect() {
    dock.openInspect();
  }

  function ask() {
    dock.openAsk(state.value.askScope, { restart: true });
  }

  function connect() {
    const [source, target] = state.value.content.filter(isSpaceSemanticNode);
    if (source && target) runCanvasAction(() => store.createRelation(source.id, target.id, "related"));
  }

  function focus() {
    store.canvasFocusIds = state.value.content.map(({ id }) => id);
  }

  function duplicate() {
    const editor = editorSession.activeEditor.value;
    if (editor) runCanvasAction(() => editor.duplicateSelection());
  }

  function align(mode: CanvasAlignMode) {
    const editor = editorSession.activeEditor.value;
    if (editor) runCanvasAction(() => editor.alignSelection(mode));
  }

  function distribute(mode: CanvasDistributeMode) {
    const editor = editorSession.activeEditor.value;
    if (editor) runCanvasAction(() => editor.distributeSelection(mode));
  }

  function autoLayout(direction: CanvasAutoLayoutDirection) {
    const editor = editorSession.activeEditor.value;
    if (editor) runCanvasAction(() => editor.autoLayoutSelection(direction));
  }

  function moveLayer(mode: CanvasLayerMove) {
    editorSession.activeEditor.value?.moveSelectionLayer(mode);
  }

  function toggleLock() {
    runCanvasAction(() =>
      store.executeSpaceEdit({
        kind: "structure.set-lock",
        rootIds: state.value.structureNodeIds,
        locked: !state.value.allLocked,
      }),
    );
  }

  function moveToSpace(spaceId: string) {
    runCanvasAction(() => store.moveStructureNodes(state.value.structureNodeIds, spaceId));
  }

  function remove() {
    const editor = editorSession.activeEditor.value;
    if (editor) runCanvasAction(() => editor.deleteSelection());
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
    duplicate,
    focus,
    fitSection: structureActions.fitSection,
    inspect,
    moveToSpace,
    moveLayer,
    open,
    parentTargets: reparentActions.targets,
    reparent: reparentActions.reparent,
    remove,
    selectedNodes,
    selectedAppearance,
    setAppearance,
    state,
    toggleLock,
    toggleSectionCollapsed: structureActions.toggleSectionCollapsed,
    visible,
  };
}
