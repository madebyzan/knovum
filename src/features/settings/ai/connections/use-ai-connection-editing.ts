import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { removeAiConnection, saveAiConnection } from "../../../../foundation/ai/connection-settings";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useEditorLeave } from "../../../../foundation/lifecycle/use-editor-leave";
import type { AiConnectionSettings } from "../../../../shared/contracts/generated/AiConnectionSettings";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import { useAiEditorMutation } from "../use-ai-editor-mutation";
import type { AiConnectionSubmission } from "./draft";

interface ConnectionEditingOptions {
  connections: () => readonly AiConnectionSettings[];
  protectedConnectionIds: () => readonly string[];
  providersAvailable: () => boolean;
}

export function useAiConnectionEditing(options: ConnectionEditingOptions) {
  const { t } = useI18n();
  const selectedId = shallowRef<string | null>();
  const dirty = shallowRef(false);
  const mutation = useAiEditorMutation();
  const { busy, error } = mutation;
  const expandedId = shallowRef<string>();
  const editing = computed(() => options.connections().find(({ id }) => id === selectedId.value));

  function openConnection(connectionId: string | null) {
    if (busy.value || (connectionId === null && !options.providersAvailable())) return;
    selectedId.value = connectionId;
    dirty.value = false;
    error.value = "";
  }

  function closeEditor() {
    selectedId.value = undefined;
    dirty.value = false;
    error.value = "";
  }

  const requestClose = useEditorLeave({
    pending: mutation.pending,
    dirty: () => dirty.value,
    confirmDiscard,
    close: closeEditor,
  });

  async function saveConnection(submission: AiConnectionSubmission) {
    if (busy.value) return;
    await mutation.run(async () => {
      const receipt = await saveAiConnection(submission.connection, submission.secret);
      if (receipt.status === "credential-failed") {
        error.value = `${t("settings.ai.connections.credentials.saveFailed")}: ${getErrorMessage(receipt.cause)}`;
        return false;
      }
      expandedId.value = submission.connection.id;
      closeEditor();
      return true;
    });
  }

  async function removeConnection(connection: AiConnectionSettings) {
    if (busy.value || options.protectedConnectionIds().includes(connection.id)) return;
    if (
      !(await confirmAction({
        title: t("common.delete"),
        description: t("settings.ai.connections.delete.title", { name: connection.name }),
        confirmLabel: t("common.delete"),
        danger: true,
      }))
    )
      return;
    await mutation.run(async () => {
      await removeAiConnection(connection.id);
      if (selectedId.value === connection.id) closeEditor();
      return true;
    });
  }

  function confirmDiscard() {
    return confirmAction({
      title: t("settings.ai.editor.discard.title"),
      description: t("settings.ai.editor.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    });
  }
  return {
    selectedId,
    dirty,
    busy,
    error,
    expandedId,
    editing,
    openConnection,
    requestClose,
    saveConnection,
    removeConnection,
  };
}
