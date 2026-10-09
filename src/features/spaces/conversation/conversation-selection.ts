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
  restoreDraft: (id: string) => Promise<boolean>;
  clearBranches: () => void;
  loadBranches: (id: string) => Promise<void>;
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

  async function loadSelection(id: string, branchId: string) {
    const requestedSpaceId = input.spaceId.value;
    const page = await listConversationMessages(id, branchId);
    if (input.selectedId.value !== id || input.spaceId.value !== requestedSpaceId) return;
    input.messages.value = page;
    await input.loadBranches(id);
    if (input.selectedId.value !== id || input.spaceId.value !== requestedSpaceId) return;
    input.hasOlder.value = page.length === 100;
  }

  async function selectConversation(id: string, skipRemember = false, acceptedBranchId?: string) {
    if (!skipRemember) await input.rememberDraft();
    const conversation =
      input.conversations.value.find((item) => item.id === id) ??
      (input.selectedId.value === id ? input.selectedSnapshot.value : undefined);
    input.selectedId.value = id;
    input.selectedSnapshot.value = conversation;
    await input.restoreDraft(id);
    if (!id) {
      input.messages.value = [];
      input.clearBranches();
      input.hasOlder.value = false;
      return;
    }
    const branchId = conversation?.branchId ?? acceptedBranchId;
    if (branchId) await loadSelection(id, branchId);
  }

  return { refreshConversations, selectConversation };
}
