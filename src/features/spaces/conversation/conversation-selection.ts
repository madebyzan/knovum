import type { Ref } from "vue";

import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import { listConversations, listConversationMessages } from "./conversation-ipc";

interface ConversationSelectionInput {
  spaceId: Ref<string | undefined>;
  query: Ref<string>;
  showingArchived: Ref<boolean>;
  conversations: Ref<Conversation[]>;
  selectedId: Ref<string>;
  selectedSnapshot: Ref<Conversation | undefined>;
  messages: Ref<ConversationMessage[]>;
  hasOlder: Ref<boolean>;
  rememberDraft: () => Promise<void>;
  restoreDraft: (conversationId: string) => Promise<boolean>;
  clearBranches: () => void;
  loadBranches: (conversationId: string) => Promise<void>;
}

export interface ConversationSelectionOptions {
  /** Defaults to true; false when the current draft has already moved to the selected Conversation. */
  rememberDraft?: boolean;
  /** Branch accepted by the backend before the Conversation list knows about it. */
  branchId?: string;
}

export function createConversationSelection(input: ConversationSelectionInput) {
  async function refreshConversations() {
    const requestedSpaceId = input.spaceId.value;
    if (!requestedSpaceId) return;
    const requestedQuery = input.query.value;
    const requestedArchived = input.showingArchived.value;
    const loaded = await listConversations(requestedSpaceId, requestedQuery, requestedArchived);
    if (
      input.spaceId.value !== requestedSpaceId ||
      input.query.value !== requestedQuery ||
      input.showingArchived.value !== requestedArchived
    )
      return;
    input.conversations.value = loaded;
    input.selectedSnapshot.value = input.selectedId.value
      ? (loaded.find(({ id }) => id === input.selectedId.value) ?? input.selectedSnapshot.value)
      : undefined;
  }

  async function loadSelection(conversationId: string, branchId: string) {
    const requestedSpaceId = input.spaceId.value;
    const page = await listConversationMessages(conversationId, branchId);
    if (input.selectedId.value !== conversationId || input.spaceId.value !== requestedSpaceId) return;
    input.messages.value = page;
    await input.loadBranches(conversationId);
    if (input.selectedId.value !== conversationId || input.spaceId.value !== requestedSpaceId) return;
    input.hasOlder.value = page.length === 100;
  }

  async function selectConversation(conversationId: string, options: ConversationSelectionOptions = {}) {
    if (options.rememberDraft !== false) await input.rememberDraft();
    const conversation =
      input.conversations.value.find((item) => item.id === conversationId) ??
      (input.selectedId.value === conversationId ? input.selectedSnapshot.value : undefined);
    input.selectedId.value = conversationId;
    input.selectedSnapshot.value = conversation;
    await input.restoreDraft(conversationId);
    if (!conversationId) {
      input.messages.value = [];
      input.clearBranches();
      input.hasOlder.value = false;
      return;
    }
    const branchId = conversation?.branchId ?? options.branchId;
    if (branchId) await loadSelection(conversationId, branchId);
  }

  return { refreshConversations, selectConversation };
}
