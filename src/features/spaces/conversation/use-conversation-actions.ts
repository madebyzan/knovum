import { ask, save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import type { ComputedRef, ShallowRef } from "vue";

import {
  archiveAssistantConversation,
  branchAssistantConversation,
  deleteAssistantConversation,
  exportAssistantConversation,
  renameAssistantConversation,
  type ConversationSummary,
  type AssistantMessage,
} from "./conversation-ipc";

interface ConversationActionState {
  selected: ComputedRef<ConversationSummary | undefined>;
  selectedId: ShallowRef<string>;
  messages: ShallowRef<AssistantMessage[]>;
  loading: ShallowRef<boolean>;
  prompt: ShallowRef<string>;
  refresh: () => Promise<void>;
  select: (id: string) => Promise<void>;
  send: () => Promise<void>;
}

export function useConversationActions(state: ConversationActionState) {
  async function renameSelected(title: string) {
    const conversation = state.selected.value;
    const nextTitle = title.trim();
    if (!conversation || !nextTitle) return;
    await renameAssistantConversation(conversation.id, nextTitle);
    await state.refresh();
  }

  async function archiveSelected(archived = true) {
    const conversation = state.selected.value;
    if (!conversation) return;
    await archiveAssistantConversation(conversation.id, archived);
    clearSelection();
    await state.refresh();
  }

  async function deleteSelected() {
    const conversation = state.selected.value;
    if (!conversation) return;
    const confirmed = await ask(`Delete “${conversation.title}” and its task history?`, {
      kind: "warning",
      title: "Delete conversation",
    });
    if (!confirmed) return;
    await deleteAssistantConversation(conversation.id);
    clearSelection();
    await state.refresh();
  }

  async function exportSelected() {
    const conversation = state.selected.value;
    if (!conversation?.branchId) return;
    const destination = await save({ defaultPath: `${conversation.title}.json` });
    if (!destination) return;
    const value = await exportAssistantConversation(conversation.id, conversation.branchId);
    await writeTextFile(destination, `${JSON.stringify(value, null, 2)}\n`);
  }

  async function branchFrom(message: AssistantMessage) {
    const conversation = state.selected.value;
    if (!conversation?.branchId || state.loading.value) return;
    const branchId = await branchAssistantConversation(conversation.id, conversation.branchId, message.id);
    await adoptBranch(conversation.id, branchId);
  }

  async function regenerate(message: AssistantMessage) {
    const conversation = state.selected.value;
    const index = state.messages.value.findIndex(({ id }) => id === message.id);
    const source = state.messages.value.slice(0, index).findLast(({ role }) => role === "user");
    if (!conversation?.branchId || !source || state.loading.value) return;
    const text = source.content.parts
      .filter((part) => part.type === "text")
      .map((part) => part.text)
      .join("\n");
    const branchId = await branchAssistantConversation(
      conversation.id,
      conversation.branchId,
      source.id,
      false,
    );
    await adoptBranch(conversation.id, branchId);
    state.prompt.value = text;
    await state.send();
  }

  function clearSelection() {
    state.selectedId.value = "";
    state.messages.value = [];
  }

  async function adoptBranch(conversationId: string, branchId: string) {
    await state.refresh();
    const conversation = state.selected.value;
    if (conversation?.id === conversationId) conversation.branchId = branchId;
    await state.select(conversationId);
  }

  return { renameSelected, archiveSelected, deleteSelected, exportSelected, branchFrom, regenerate };
}
