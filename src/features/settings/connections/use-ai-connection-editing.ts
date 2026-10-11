import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { removeAiConnection, saveAiConnection } from "../../../foundation/ai/connection-settings";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { AiConnectionSettings } from "../../../shared/contracts/generated/AiConnectionSettings";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { useSettingsLeaveDecision } from "../use-settings-leave-decision";
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
  const busy = shallowRef(false);
  const error = shallowRef("");
  const expandedId = shallowRef<string>();
  const editing = computed(() => options.connections().find(({ id }) => id === selectedId.value));
  let admittedMutation: Promise<boolean> | undefined;

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

  async function requestClose() {
    if (admittedMutation) {
      const succeeded = await admittedMutation;
      if (!succeeded) return false;
      if (selectedId.value === undefined) return true;
    }
    if (dirty.value && !(await confirmDiscard())) return false;
    closeEditor();
    return true;
  }
  useSettingsLeaveDecision({
    async prepare() {
      if (admittedMutation && !(await admittedMutation)) return false;
      return !dirty.value || confirmDiscard();
    },
    finalize: closeEditor,
  });

  async function saveConnection(submission: AiConnectionSubmission) {
    if (busy.value) return;
    admittedMutation = runMutation(async () => {
      const receipt = await saveAiConnection(submission.connection, submission.secret);
      if (receipt.status === "credential-failed") {
        error.value = `${t("settings.ai.connections.credentials.saveFailed")}: ${getErrorMessage(receipt.cause)}`;
        return false;
      }
      expandedId.value = submission.connection.id;
      closeEditor();
      return true;
    });
    await admittedMutation;
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
    admittedMutation = runMutation(async () => {
      await removeAiConnection(connection.id);
      if (selectedId.value === connection.id) closeEditor();
      return true;
    });
    await admittedMutation;
  }

  function runMutation(mutation: () => Promise<boolean>) {
    busy.value = true;
    error.value = "";
    const admitted: Promise<boolean> = mutation()
      .catch((cause) => {
        error.value = `${t("settings.general.saveFailed")}: ${getErrorMessage(cause)}`;
        return false;
      })
      .finally(() => {
        busy.value = false;
        if (admittedMutation === admitted) admittedMutation = undefined;
      });
    return admitted;
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
