import { shallowRef, type Ref } from "vue";

import type { AssistantConversation } from "../contracts/generated/AssistantConversation";
import { activateAssistantBranch, listAssistantBranches } from "./conversation-ipc";

export function useConversationBranches(input: {
  selected: Ref<AssistantConversation | undefined>;
  selectedId: Ref<string>;
  loading: Ref<boolean>;
  refresh: () => Promise<void>;
  select: (id: string) => Promise<void>;
}) {
  const branches = shallowRef<string[]>([]);

  async function load(id: string) {
    const loaded = id ? await listAssistantBranches(id) : [];
    if (input.selectedId.value === id) branches.value = loaded;
  }

  async function selectBranch(branchId: string) {
    const conversation = input.selected.value;
    if (!conversation || input.loading.value || conversation.branchId === branchId) return;
    await activateAssistantBranch(conversation.id, branchId);
    await input.refresh();
    await input.select(conversation.id);
  }

  return { branches, load, selectBranch };
}
