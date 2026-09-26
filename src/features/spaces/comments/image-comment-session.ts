import { defineStore } from "pinia";
import { readonly, shallowRef, watch } from "vue";

import { useSpaceStore } from "../session/space-store";

export type ImageCommentMode = "point" | "draw" | "region";

export const useImageCommentSession = defineStore("spaces-image-comments", () => {
  const space = useSpaceStore();
  const activeNodeId = shallowRef<string>();
  const mode = shallowRef<ImageCommentMode>("point");
  const conversationDraft = shallowRef<{ nodeId: string; text: string }>();
  function begin(nodeId: string) {
    activeNodeId.value = nodeId;
    mode.value = "point";
  }
  function cancel() {
    activeNodeId.value = undefined;
  }
  function setMode(value: ImageCommentMode) {
    mode.value = value;
  }
  function addToConversation(nodeId: string, text: string) {
    conversationDraft.value = { nodeId, text };
  }
  function takeConversationDraft() {
    const draft = conversationDraft.value;
    conversationDraft.value = undefined;
    return draft;
  }
  watch(
    () => space.activeSpaceId,
    () => {
      cancel();
      conversationDraft.value = undefined;
    },
    { flush: "sync" },
  );
  watch(
    () => space.nodes,
    (nodes) => {
      const ids = new Set(nodes.map(({ id }) => id));
      if (activeNodeId.value && !ids.has(activeNodeId.value)) cancel();
      if (conversationDraft.value && !ids.has(conversationDraft.value.nodeId))
        conversationDraft.value = undefined;
    },
    { flush: "sync" },
  );

  return {
    activeNodeId: readonly(activeNodeId),
    mode: readonly(mode),
    conversationDraft: readonly(conversationDraft),
    begin,
    cancel,
    setMode,
    addToConversation,
    takeConversationDraft,
  };
});
