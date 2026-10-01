import type { Ref } from "vue";

import type { AssistantConversation } from "../contracts/generated/AssistantConversation";
import type { AssistantMessage } from "../contracts/generated/AssistantMessage";
import { listAssistantConversations, listAssistantMessages } from "./conversation-ipc";

export function createConversationSelection(input: {
  spaceId: Ref<string | undefined>;
  query: Ref<string>;
  showingArchived: Ref<boolean>;
  conversations: Ref<AssistantConversation[]>;
  selectedId: Ref<string>;
  selectedSnapshot: Ref<AssistantConversation | undefined>;
  messages: Ref<AssistantMessage[]>;
  hasOlder: Ref<boolean>;
  rememberDraft: () => void;
  restoreDraft: (id: string) => void;
  clearBranches: () => void;
  loadBranches: (id: string) => Promise<void>;
}) {
  async function refreshConversations() {
    const requestedSpaceId = input.spaceId.value;
    if (!requestedSpaceId) return;
    const requestedQuery = input.query.value;
    const requestedArchived = input.showingArchived.value;
    const loaded = await listAssistantConversations(requestedSpaceId, requestedQuery, requestedArchived);
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
    const page = await listAssistantMessages(id, branchId);
    if (input.selectedId.value !== id || input.spaceId.value !== requestedSpaceId) return;
    input.messages.value = page;
    await input.loadBranches(id);
    if (input.selectedId.value !== id || input.spaceId.value !== requestedSpaceId) return;
    input.hasOlder.value = page.length === 100;
  }

  async function selectConversation(id: string, skipRemember = false, acceptedBranchId?: string) {
    if (!skipRemember) input.rememberDraft();
    const conversation =
      input.conversations.value.find((item) => item.id === id) ??
      (input.selectedId.value === id ? input.selectedSnapshot.value : undefined);
    input.selectedId.value = id;
    input.selectedSnapshot.value = conversation;
    input.restoreDraft(id);
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
