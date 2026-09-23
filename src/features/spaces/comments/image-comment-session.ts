import { defineStore } from "pinia";
import { readonly, shallowRef } from "vue";

export type ImageCommentMode = "point" | "draw" | "region";

export const useImageCommentSession = defineStore("spaces-image-comments", () => {
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
