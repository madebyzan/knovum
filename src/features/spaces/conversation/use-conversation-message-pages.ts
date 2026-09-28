import { shallowRef, type Ref, type ShallowRef } from "vue";

import {
  listAssistantMessages,
  type AssistantConversation,
  type AssistantMessage,
} from "./conversation-ipc";

export function useConversationMessagePages(input: {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  selectedConversation: Ref<AssistantConversation | undefined>;
  messages: ShallowRef<AssistantMessage[]>;
}) {
  const loadingOlder = shallowRef(false);
  const hasOlder = shallowRef(false);
  const scrollPositions = shallowRef<Record<string, number>>({});

  async function loadOlder() {
    const conversation = input.selectedConversation.value;
    const first = input.messages.value[0];
    if (!conversation?.branchId || !first || loadingOlder.value || !hasOlder.value) return;
    loadingOlder.value = true;
    try {
      const requestedSpaceId = input.spaceId.value;
      const page = await listAssistantMessages(conversation.id, conversation.branchId, first);
      if (input.spaceId.value !== requestedSpaceId || input.selectedConversationId.value !== conversation.id) return;
      input.messages.value = [...page, ...input.messages.value];
      hasOlder.value = page.length === 100;
    } finally {
      loadingOlder.value = false;
    }
  }

  function rememberScroll(id: string, top: number) {
    scrollPositions.value = { ...scrollPositions.value, [id]: top };
  }

  return { loadingOlder, hasOlder, scrollPositions, loadOlder, rememberScroll };
}
