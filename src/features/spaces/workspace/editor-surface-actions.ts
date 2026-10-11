import { nextTick } from "vue";

import { runCanvasSessionTransition } from "../canvas/editor-session";
import { isSpaceEditorNode } from "../canvas/node-capabilities";
import { prepareEditorTabsClose } from "./editor-tab-lifecycle";
import type { SpaceSessionContext } from "./space-session-context";
import { activateSpaceWorkItem, transitionActiveSpaceSession } from "./space-transition";
import { removeSpaceWorkItems, selectSpaceWorkItem, type SpaceEditorTab } from "./work-items";

export function createEditorSurfaceActions(context: SpaceSessionContext) {
  async function openEditor(kind: SpaceEditorTab["kind"], nodeId: string) {
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("Space requires an active space");
    await transitionActiveSpaceSession(context, async () => {
      const node = context.nodes.value.find(({ id }) => id === nodeId);
      if (context.activeSpaceId.value !== spaceId || !node) throw new Error("Editor content is unavailable");
      if (kind === "document" ? !isSpaceEditorNode(node) : node.data.kind !== "video")
        throw new Error("Content does not support this editor");
      let tab = context.editorTabs.value.find(
        (item) => item.spaceId === spaceId && item.kind === kind && item.nodeId === nodeId,
      );
      if (!tab) {
        tab = { id: crypto.randomUUID(), spaceId, kind, nodeId };
        context.editors.open(tab, node);
        context.editorTabs.value = [...context.editorTabs.value, tab];
      }
      selectSpaceWorkItem(context, spaceId, tab.id);
      await nextTick();
    });
  }
  async function closeEditorTab(tabId: string) {
    const tab = context.editorTabs.value.find(({ id }) => id === tabId);
    if (!tab) return;
    await runCanvasSessionTransition(async () => {
      const finishClose = await prepareEditorTabsClose(context, [tab]);
      await finishClose();
    });
  }
  return {
    openDocumentEditor: (nodeId: string) => openEditor("document", nodeId),
    openVideoEditor: (nodeId: string) => openEditor("video", nodeId),
    activateEditorTab: (tabId: string) => activateSpaceWorkItem(context, tabId),
    closeEditorTab,
    discardUnavailableEditorTab: async (tabId: string) => {
      const tab = context.editorTabs.value.find(({ id }) => id === tabId);
      if (!tab || !context.editors.canDiscard(tab))
        throw new Error("Editor draft cannot be discarded without a resource error");
      removeSpaceWorkItems(context, new Set([tab.id]));
      await nextTick();
      context.editors.release(tab);
    },
    returnToCanvas: () => activateSpaceWorkItem(context),
  };
}
