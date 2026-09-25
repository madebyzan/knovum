import { watch, type Ref } from "vue";

import { useImageCommentSession } from "../comments/image-comment-session";
import { useSpaceStore } from "../session/space-store";

export function useImageCommentDraft(spaceId: Ref<string | undefined>, prompt: Ref<string>) {
  const store = useSpaceStore();
  const comments = useImageCommentSession();
  watch(
    () => comments.conversationDraft,
    (draft) => {
      if (!draft || !spaceId.value || !store.nodes.some(({ id }) => id === draft.nodeId)) return;
      prompt.value = [prompt.value.trim(), draft.text].filter(Boolean).join("\n\n");
      comments.takeConversationDraft();
    },
    { immediate: true },
  );
}
