import type { ComputedRef, ShallowRef } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { exportFilter, safeExportFileName } from "../assets/export/naming";
import { saveSpaceTextFile } from "../assets/export/text";
import type { Conversation } from "../contracts/generated/Conversation";
import type { ConversationMessage } from "../contracts/generated/ConversationMessage";
import {
  archiveConversation,
  unarchiveConversation,
  branchConversation,
  deleteConversation,
  loadConversationBranchMessages,
  renameConversation,
} from "./conversation-ipc";

interface ConversationActionState {
  selected: ComputedRef<Conversation | undefined>;
  selectedId: ShallowRef<string>;
  messages: ShallowRef<ConversationMessage[]>;
  loading: ShallowRef<boolean>;
  prompt: ShallowRef<string>;
  refresh: () => Promise<void>;
  select: (conversationId: string) => Promise<void>;
  send: () => Promise<void>;
}

export function createConversationActions(state: ConversationActionState) {
  async function renameSelected(title: string) {
    const conversation = state.selected.value;
    const nextTitle = title.trim();
    if (!conversation || !nextTitle) return;
    await renameConversation(conversation.id, nextTitle);
    await state.refresh();
  }

  async function archiveSelected() {
    const conversation = state.selected.value;
    if (!conversation) return;
    await archiveConversation(conversation.id);
    clearSelection();
    await state.refresh();
  }

  async function unarchiveSelected() {
    const conversation = state.selected.value;
    if (!conversation) return;
    await unarchiveConversation(conversation.id);
    clearSelection();
    await state.refresh();
  }

  async function deleteSelected() {
    const conversation = state.selected.value;
    if (!conversation) return;
    const confirmed = await confirmAction({
      title: i18n.global.t("common.delete"),
      description: i18n.global.t("spaces.conversation.conversations.deleteConfirm", {
        title: conversation.title,
      }),
      confirmLabel: i18n.global.t("common.delete"),
      danger: true,
    });
    if (!confirmed) return;
    await deleteConversation(conversation.id);
    clearSelection();
    await state.refresh();
  }

  async function exportSelected() {
    const conversation = state.selected.value;
    if (!conversation?.branchId) return;
    try {
      const value = await loadConversationBranchMessages(conversation.id, conversation.branchId);
      await saveSpaceTextFile({
        defaultPath: `${safeExportFileName(conversation.title) || "conversation"}.json`,
        filters: [exportFilter("json")],
        text: `${JSON.stringify(value, null, 2)}\n`,
      });
    } catch (cause) {
      toast.error(
        `${i18n.global.t("spaces.conversation.conversations.exportFailed")}: ${getErrorMessage(cause)}`,
      );
    }
  }

  async function branchFrom(message: ConversationMessage) {
    const conversation = state.selected.value;
    if (!conversation?.branchId || state.loading.value) return;
    try {
      const branchId = await branchConversation({
        conversationId: conversation.id,
        sourceBranchId: conversation.branchId,
        throughMessageId: message.id,
      });
      await adoptBranch(conversation.id, branchId);
      toast.success(i18n.global.t("spaces.conversation.conversations.branchCreated"));
    } catch (cause) {
      toast.error(getErrorMessage(cause));
    }
  }

  async function regenerate(message: ConversationMessage) {
    const conversation = state.selected.value;
    const index = state.messages.value.findIndex(({ id }) => id === message.id);
    const source = state.messages.value.slice(0, index).findLast(({ role }) => role === "user");
    if (!conversation?.branchId || !source || state.loading.value) return;
    if (source.content.attachments?.length) {
      toast.error(i18n.global.t("spaces.conversation.conversations.regenerateAttachments"));
      return;
    }
    const text = source.content.parts
      .filter((part) => part.type === "text")
      .map((part) => part.text)
      .join("\n");
    try {
      const branchId = await branchConversation({
        conversationId: conversation.id,
        sourceBranchId: conversation.branchId,
        throughMessageId: source.id,
        includeBoundary: false,
      });
      await adoptBranch(conversation.id, branchId);
      state.prompt.value = text;
      await state.send();
    } catch (cause) {
      toast.error(getErrorMessage(cause));
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
