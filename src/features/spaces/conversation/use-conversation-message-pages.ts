import { shallowRef, type Ref, type ShallowRef } from "vue";

import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import { listConversationMessages } from "./conversation-ipc";

interface ConversationMessagePagesInput {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  selectedConversation: Ref<Conversation | undefined>;
  messages: ShallowRef<ConversationMessage[]>;
}

export function useConversationMessagePages(input: ConversationMessagePagesInput) {
  const loadingOlder = shallowRef(false);
  const hasOlder = shallowRef(false);
  const scrollTopByConversationId = shallowRef<Record<string, number>>({});

  async function loadOlder() {
    const conversation = input.selectedConversation.value;
    const first = input.messages.value[0];
    if (!conversation?.branchId || !first || loadingOlder.value || !hasOlder.value) return;
    loadingOlder.value = true;
    try {
      const requestedSpaceId = input.spaceId.value;
      const page = await listConversationMessages(conversation.id, conversation.branchId, first);
      if (input.spaceId.value !== requestedSpaceId || input.selectedConversationId.value !== conversation.id)
        return;
      input.messages.value = [...page, ...input.messages.value];
      hasOlder.value = page.length === 100;
    } finally {
      loadingOlder.value = false;
    }
  }

  function rememberScroll(conversationId: string, scrollTop: number) {
    scrollTopByConversationId.value = { ...scrollTopByConversationId.value, [conversationId]: scrollTop };
  }

  return { loadingOlder, hasOlder, scrollTopByConversationId, loadOlder, rememberScroll };
}
