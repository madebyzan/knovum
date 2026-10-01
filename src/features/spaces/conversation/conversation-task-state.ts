import type { Ref, ShallowRef } from "vue";

import type { AssistantTaskAccepted } from "../contracts/generated/AssistantTaskAccepted";

type Selection = { spaceId: string; conversationId: string };

interface ConversationTaskStateInput {
  spaceId: Ref<string | undefined>;
  selectedConversationId: Ref<string>;
  activeConversationId: ShallowRef<string>;
  refresh: () => Promise<void>;
  select: (id: string, skipRemember?: boolean, branchId?: string) => Promise<void>;
  moveDraft: (fromId: string, toId: string) => void;
}

export function createConversationTaskState(input: ConversationTaskStateInput) {
  function isSelected(selection: Selection) {
    return (
      input.spaceId.value === selection.spaceId &&
      input.selectedConversationId.value === selection.conversationId
    );
  }

  async function accepted(task: AssistantTaskAccepted, selection: Selection) {
    input.activeConversationId.value = task.conversationId;
    if (!isSelected(selection)) return;
    await input.refresh();
    if (!isSelected(selection)) return;
    if (!selection.conversationId) input.moveDraft("", task.conversationId);
    await input.select(task.conversationId, !selection.conversationId, task.branchId);
  }

  async function settled(task: AssistantTaskAccepted, spaceId: string) {
    if (input.spaceId.value !== spaceId) return;
    await input.refresh();
    if (input.spaceId.value === spaceId && input.selectedConversationId.value === task.conversationId)
      await input.select(task.conversationId, false, task.branchId);
  }

  return { accepted, settled };
}
