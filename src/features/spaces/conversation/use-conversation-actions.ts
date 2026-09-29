import type { ComputedRef, ShallowRef } from "vue";
import { toast } from "vue-sonner";
import { i18n } from "../../../foundation/i18n/i18n";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { exportSpaceTextFile, safeExportFileName } from "../assets/export";

import {
  archiveAssistantConversation,
  unarchiveAssistantConversation,
  branchAssistantConversation,
  deleteAssistantConversation,
  loadAssistantBranchMessages,
  renameAssistantConversation,
  type AssistantConversation,
  type AssistantMessage,
} from "./conversation-ipc";

interface ConversationActionState {
  selected: ComputedRef<AssistantConversation | undefined>;
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

  async function archiveSelected() {
    const conversation = state.selected.value;
    if (!conversation) return;
    await archiveAssistantConversation(conversation.id);
    clearSelection();
    await state.refresh();
  }

  async function unarchiveSelected() {
    const conversation = state.selected.value;
    if (!conversation) return;
    await unarchiveAssistantConversation(conversation.id);
    clearSelection();
    await state.refresh();
  }

  async function deleteSelected() {
    const conversation = state.selected.value;
    if (!conversation) return;
    const confirmed = await confirmAction({
      title: i18n.global.t("spaces.assistant.conversations.delete"),
      description: i18n.global.t("spaces.assistant.conversations.deleteConfirm", { title: conversation.title }),
      confirmLabel: i18n.global.t("common.delete"),
      danger: true,
    });
    if (!confirmed) return;
    await deleteAssistantConversation(conversation.id);
    clearSelection();
    await state.refresh();
  }

  async function exportSelected() {
    const conversation = state.selected.value;
    if (!conversation?.branchId) return;
    try {
      const value = await loadAssistantBranchMessages(conversation.id, conversation.branchId);
      await exportSpaceTextFile({
        defaultPath: `${safeExportFileName(conversation.title) || "conversation"}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
        text: `${JSON.stringify(value, null, 2)}\n`,
      });
    } catch (cause) {
      toast.error(`${i18n.global.t("spaces.assistant.conversations.exportFailed")}: ${String(cause)}`);
    }
  }

  async function branchFrom(message: AssistantMessage) {
    const conversation = state.selected.value;
    if (!conversation?.branchId || state.loading.value) return;
    try {
      const branchId = await branchAssistantConversation(conversation.id, conversation.branchId, message.id);
      await adoptBranch(conversation.id, branchId);
      toast.success(i18n.global.t("spaces.assistant.conversations.branchCreated"));
    } catch (cause) {
      toast.error(String(cause));
    }
  }

  async function regenerate(message: AssistantMessage) {
    const conversation = state.selected.value;
    const index = state.messages.value.findIndex(({ id }) => id === message.id);
    const source = state.messages.value.slice(0, index).findLast(({ role }) => role === "user");
    if (!conversation?.branchId || !source || state.loading.value) return;
    if (source.content.attachments?.length) {
      toast.error(i18n.global.t("spaces.assistant.conversations.regenerateAttachments"));
      return;
    }
    const text = source.content.parts
      .filter((part) => part.type === "text")
      .map((part) => part.text)
      .join("\n");
    try {
      const branchId = await branchAssistantConversation(
        conversation.id,
        conversation.branchId,
        source.id,
        false,
      );
      await adoptBranch(conversation.id, branchId);
      state.prompt.value = text;
      await state.send();
    } catch (cause) {
      toast.error(String(cause));
    }
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

  return {
    renameSelected,
    archiveSelected,
    unarchiveSelected,
    deleteSelected,
    exportSelected,
    branchFrom,
    regenerate,
  };
}
