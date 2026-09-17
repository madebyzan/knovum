import { computed, nextTick } from "vue";
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
  const state = computed(() => canvasSelectionProductState(store.nodes, selection.value.nodeIds));
  const { t } = useI18n();
  const visible = computed(
    () => selection.value.nodeIds.length > 0 || selection.value.relationIds.length > 0,
  );
  const selectedAppearance = computed(() => selection.value.singleNodeAppearance);

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
    const section = state.value.section;
    if (section) store.focusSection(section.id);
    else if (state.value.content[0]) store.focusNeighborhood(state.value.content[0].id);
  }

  function duplicate() {
    const editor = editorSession.activeEditor.value;
    if (editor) runCanvasAction(() => editor.duplicateSelection());
  }

  function createSection() {
    const editor = editorSession.activeEditor.value;
    const bounds = editor?.getSelectionBounds();
    if (!editor || !bounds) return;
    runCanvasAction(async () => {
      const section = await store.createSection(
        state.value.content.map(({ id }) => id),
        bounds,
        t("spaces.canvas.untitledSection"),
      );
      if (!section) return;
      editor.selectNodes([section.id]);
      await nextTick();
      await editor.fitSection(section.id);
    });
  }

  function ungroup() {
    runCanvasAction(() =>
      store.assignSections(
        state.value.content.filter(({ data }) => data.sectionId).map(({ id }) => ({ id })),
      ),
    );
  }

  function fitSection() {
    const section = state.value.section;
    const editor = editorSession.activeEditor.value;
    if (section && editor) runCanvasAction(() => editor.fitSection(section.id));
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
    runCanvasAction(() => store.setLocked(selection.value.nodeIds, !state.value.allLocked));
  }

  function removeSection() {
    const section = state.value.section;
    if (section) runCanvasAction(() => store.removeSection(section.id));
  }

  function moveToSpace(spaceId: string) {
    runCanvasAction(() =>
      store.moveNodes(
        selectedNodes.value.filter((node) => node.data.kind !== "section").map((node) => node.id),
        spaceId,
      ),
    );
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
    createSection,
    distribute,
    duplicate,
    focus,
    fitSection,
    inspect,
    moveToSpace,
    moveLayer,
    open,
    remove,
    removeSection,
    selectedNodes,
    selectedAppearance,
    setAppearance,
    state,
    toggleLock,
    ungroup,
    visible,
  };
}
