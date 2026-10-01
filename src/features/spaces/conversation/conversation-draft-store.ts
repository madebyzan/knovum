import { defineStore } from "pinia";
import { shallowRef } from "vue";

import type { SubmitAssistantTaskRequest } from "../contracts/generated/SubmitAssistantTaskRequest";

type Attachment = NonNullable<SubmitAssistantTaskRequest["attachments"]>[number];
interface ConversationDraft {
  prompt: string;
  attachments: Attachment[];
}

export const useConversationDraftStore = defineStore("spaces-conversation-drafts", () => {
  const drafts = shallowRef<Record<string, ConversationDraft>>({});

  function read(spaceId: string, conversationId: string): ConversationDraft | undefined {
    const draft = drafts.value[`${spaceId}:${conversationId}`];
    return draft && { prompt: draft.prompt, attachments: [...draft.attachments] };
  }

  function save(spaceId: string, conversationId: string, draft: ConversationDraft) {
    drafts.value = {
      ...drafts.value,
      [`${spaceId}:${conversationId}`]: {
        prompt: draft.prompt,
        attachments: [...draft.attachments],
      },
    };
  }

  function move(spaceId: string, fromId: string, toId: string, draft: ConversationDraft) {
    const next = { ...drafts.value };
    delete next[`${spaceId}:${fromId}`];
    next[`${spaceId}:${toId}`] = { prompt: draft.prompt, attachments: [...draft.attachments] };
    drafts.value = next;
  }

  function remove(spaceId: string, conversationId: string) {
    const next = { ...drafts.value };
    delete next[`${spaceId}:${conversationId}`];
    drafts.value = next;
  }

  return { read, save, move, remove };
});
