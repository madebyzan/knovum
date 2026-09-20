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
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { openSpaceNode } from "../../canvas/selection-actions";
import { useContextDock } from "../../canvas/use-context-dock";
import type { StructureCommand } from "../../organization/structure/commands/contract";
import { useSpaceStore } from "../../session/space-store";
import { canvasSelectionProductState } from "./selection-product-state";

function run(operation: () => Promise<unknown>) {
  void Promise.resolve()
    .then(operation)
    .catch((error) => toast.error(String(error)));
}

export function useCanvasSelectionActions() {
  const store = useSpaceStore();
  const dock = useContextDock();
  const session = useCanvasEditorSession();
  const { t } = useI18n();
  const structure = computed(() => session.document.value?.structure ?? { version: 1 as const, items: [] });
  const state = computed(() =>
    canvasSelectionProductState(store.nodes, session.selection.value.nodeIds, structure.value),
  );
  const selectedNodes = computed(() => state.value.directContent);
  const selectedAppearance = computed(() => session.selection.value.singleNodeAppearance);
  const visible = computed(
    () => state.value.roots.length > 0 || session.selection.value.relationIds.length > 0,
  );
  const ids = () => state.value.roots.map((item) => item.id);
  function command(command: StructureCommand) {
    const editor = session.activeEditor.value;
    if (editor) run(() => editor.executeStructureCommand(command));
  }
  function createSection() {
    const editor = session.activeEditor.value;
    if (!editor || !state.value.canSection) return;
    const id = crypto.randomUUID();
    run(async () => {
      await editor.executeStructureCommand({
        kind: "section-around-selection",
        id,
        title: t("spaces.structure.untitledSection"),
        ids: ids(),
      });
      editor.requestContainerRename(id);
    });
  }
  function group() {
    command({
      kind: "group-selection",
      id: crypto.randomUUID(),
      title: t("spaces.structure.untitledGroup"),
      ids: ids(),
    });
  }
  function ungroup() {
    if (state.value.group) command({ kind: "dissolve-container", id: state.value.group.id });
  }
  function removeSection() {
    if (state.value.section) command({ kind: "dissolve-container", id: state.value.section.id });
  }
  function fitSection() {
    if (state.value.section) command({ kind: "fit-section", id: state.value.section.id });
  }
  function setAppearance(patch: Partial<CanvasNodeAppearance>) {
    const editor = session.activeEditor.value;
    const [id] = ids();
    if (editor && id && state.value.roots.length === 1) run(() => editor.setNodeAppearance(id, patch));
  }
  function connect() {
    const [source, target] = state.value.directContent;
    if (state.value.canConnect && source && target)
      run(() => store.createRelation(source.id, target.id, "related"));
  }
  function focus() {
    if (state.value.section) store.focusSection(state.value.section.id);
    else if (state.value.group) store.focusItems([state.value.group.id]);
    else if (state.value.directContent[0]) store.focusNeighborhood(state.value.directContent[0].id);
  }
  function editorAction(
    action: (editor: NonNullable<typeof session.activeEditor.value>) => Promise<unknown>,
  ) {
    const editor = session.activeEditor.value;
    if (editor) run(() => action(editor));
  }
  return {
    state,
    selectedNodes,
    selectedAppearance,
    visible,
    setAppearance,
    connect,
    focus,
    group,
    ungroup,
    createSection,
    fitSection,
    removeSection,
    inspect: () => dock.openInspect(),
    ask: () => dock.openAsk(state.value.askScope, { restart: true }),
    open: () => run(() => openSpaceNode(store, state.value.directContent[0])),
    duplicate: () => editorAction((editor) => editor.duplicateSelection()),
    remove: () => editorAction((editor) => editor.deleteSelection()),
    toggleLock: () => command({ kind: "set-items-locked", ids: ids(), locked: !state.value.allLocked }),
    moveToRoot: () => command({ kind: "reparent-items", ids: ids(), parentId: null }),
    moveToSpace: (spaceId: string) => run(() => store.moveNodes(ids(), spaceId)),
    align: (mode: CanvasAlignMode) => editorAction((editor) => editor.alignSelection(mode)),
    distribute: (mode: CanvasDistributeMode) => editorAction((editor) => editor.distributeSelection(mode)),
    autoLayout: (direction: CanvasAutoLayoutDirection) =>
      editorAction((editor) => editor.autoLayoutSelection(direction)),
    moveLayer: (mode: CanvasLayerMove) => editorAction((editor) => editor.moveSelectionLayer(mode)),
  };
}
export type CanvasSelectionActions = ReturnType<typeof useCanvasSelectionActions>;
