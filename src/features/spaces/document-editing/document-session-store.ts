import { isEqual } from "es-toolkit";
import { defineStore } from "pinia";
import { computed, markRaw, shallowRef, type Ref, type ComputedRef } from "vue";

import { isSpaceEditorNode, type SpaceEditorNode } from "../canvas/node-capabilities";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceEditorTab } from "../workspace/workbench-tab";
import { createEditingState } from "./editing-state";

export const useDocumentSessionStore = defineStore("spaces-document-sessions", () => {
  const sessions = shallowRef<ReadonlyMap<string, DocumentSession>>(new Map());
  function open(tab: SpaceEditorTab, node: SpaceEditorNode) {
    const state = createEditingState();
    state.title.value = node.data.title;
    const source = shallowRef<SpaceEditorNode>(node);
    const session = markRaw({
      tab,
      node: source,
      state,
      saving: shallowRef(false),
      error: shallowRef<"missing" | "conflict">(),
      dirty: computed(
        () =>
          state.revision.value > state.persistedRevision.value ||
          state.title.value !== source.value.data.title,
      ),
      contentVersion: shallowRef(0),
    });
    sessions.value = new Map(sessions.value).set(tab.id, session);
  }
  function synchronize(tabId: string, node: SpaceCanvasNode | undefined) {
    const session = sessions.value.get(tabId);
    if (!session || session.saving.value) return;
    if (!isSpaceEditorNode(node)) {
      session.error.value = "missing";
      return;
    }
    const before = session.node.value.data;
    const contentChanged = documentContentChanged(session.node.value, node);
    if (session.dirty.value && (contentChanged || before.title !== node.data.title)) {
      session.error.value = "conflict";
      return;
    }
    const wasDirty = session.dirty.value;
    session.node.value = node;
    session.error.value = undefined;
    if (contentChanged) session.contentVersion.value += 1;
    if (!wasDirty) session.state.title.value = node.data.title;
  }
  function release(tabId: string) {
    const next = new Map(sessions.value);
    next.delete(tabId);
    sessions.value = next;
  }
  return { sessions, open, synchronize, release };
});

export type DocumentSession = {
  tab: SpaceEditorTab;
  node: Ref<SpaceEditorNode>;
  state: ReturnType<typeof createEditingState>;
  saving: Ref<boolean>;
  error: Ref<"missing" | "conflict" | undefined>;
  dirty: ComputedRef<boolean>;
  contentVersion: Ref<number>;
};

function documentContentChanged(before: SpaceEditorNode, after: SpaceEditorNode) {
  if (before.data.kind !== after.data.kind) return true;
  if (!isEqual(before.data.tiptapContent, after.data.tiptapContent)) return true;
  return (
    before.data.kind === "markdown" && after.data.kind === "markdown" && before.data.text !== after.data.text
  );
}
