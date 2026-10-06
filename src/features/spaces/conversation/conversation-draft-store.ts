import { defineStore } from "pinia";
import { shallowRef } from "vue";

import { copyExtensionSelection } from "../../../foundation/extensions/extension-selection";
import type { AgentExtensionSelection } from "../../../shared/contracts/generated/AgentExtensionSelection";
import type { ConversationAttachment } from "../contracts/generated/ConversationAttachment";

interface ConversationDraft {
  prompt: string;
  attachments: ConversationAttachment[];
  extensions: AgentExtensionSelection;
}

export const useConversationDraftStore = defineStore("spaces-conversation-drafts", () => {
  const drafts = shallowRef<Record<string, ConversationDraft>>({});

  function read(spaceId: string, conversationId: string): ConversationDraft | undefined {
    const draft = drafts.value[`${spaceId}:${conversationId}`];
    return (
      draft && {
        prompt: draft.prompt,
        attachments: [...draft.attachments],
        extensions: copyExtensionSelection(draft.extensions),
      }
    );
  }

  function save(spaceId: string, conversationId: string, draft: ConversationDraft) {
    drafts.value = {
      ...drafts.value,
      [`${spaceId}:${conversationId}`]: {
        prompt: draft.prompt,
        attachments: [...draft.attachments],
        extensions: copyExtensionSelection(draft.extensions),
      },
    };
  }

  function move(spaceId: string, fromId: string, toId: string, draft: ConversationDraft) {
    const next = { ...drafts.value };
    delete next[`${spaceId}:${fromId}`];
    next[`${spaceId}:${toId}`] = {
      prompt: draft.prompt,
      attachments: [...draft.attachments],
      extensions: copyExtensionSelection(draft.extensions),
    };
    drafts.value = next;
  }

  function remove(spaceId: string, conversationId: string) {
    const next = { ...drafts.value };
    delete next[`${spaceId}:${conversationId}`];
    drafts.value = next;
  }

  return { read, save, move, remove };
});
