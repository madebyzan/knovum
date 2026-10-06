import { shallowRef, type Ref } from "vue";

import type { Conversation } from "../contracts/generated/Conversation";
import { activateConversationBranch, listConversationBranches } from "./conversation-ipc";

interface ConversationBranchesInput {
  selected: Ref<Conversation | undefined>;
  selectedId: Ref<string>;
  loading: Ref<boolean>;
  refresh: () => Promise<void>;
  select: (id: string) => Promise<void>;
}

export function useConversationBranches(input: ConversationBranchesInput) {
  const branches = shallowRef<string[]>([]);

  async function load(id: string) {
    const loaded = id ? await listConversationBranches(id) : [];
    if (input.selectedId.value === id) branches.value = loaded;
  }

  async function selectBranch(branchId: string) {
    const conversation = input.selected.value;
    if (!conversation || input.loading.value || conversation.branchId === branchId) return;
    await activateConversationBranch(conversation.id, branchId);
    await input.refresh();
    await input.select(conversation.id);
  }

  return { branches, load, selectBranch };
}
