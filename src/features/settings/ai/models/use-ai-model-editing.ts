import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import {
  removeAiConnectionModel,
  saveAiConnectionModel,
} from "../../../../foundation/ai/connection-settings";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { AiConnectionSettings } from "../../../../shared/contracts/generated/AiConnectionSettings";
import type { AiModelDefinition } from "../../../../shared/contracts/generated/AiModelDefinition";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import { useSettingsLeaveDecision } from "../../navigation/use-settings-leave-decision";

interface ModelEditingOptions {
  connection: () => AiConnectionSettings;
  protectedModelTargets: () => readonly string[];
}

export function useAiModelEditing(options: ModelEditingOptions) {
  const { t } = useI18n();
  const selected = shallowRef<string | null>();
  const dirty = shallowRef(false);
  const busy = shallowRef(false);
  const error = shallowRef("");
  const editingModel = computed(() => options.connection().models.find(({ id }) => id === selected.value));
  let admittedMutation: Promise<boolean> | undefined;

  function modelInUse(modelId: string) {
    return options.protectedModelTargets().includes(
      JSON.stringify({
        type: "model",
        connectionId: options.connection().id,
        modelId,
      }),
    );
  }

  function openModel(modelId?: string) {
    if (busy.value) return;
    selected.value = modelId ?? null;
    dirty.value = false;
    error.value = "";
  }

  function closeEditor() {
    selected.value = undefined;
    dirty.value = false;
    error.value = "";
  }

  async function requestClose() {
    if (admittedMutation) {
      const succeeded = await admittedMutation;
      if (!succeeded) return false;
      if (selected.value === undefined) return true;
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

  async function saveModel(model: AiModelDefinition) {
    if (busy.value) return;
    const connection = options.connection();
    if (
      connection.models.some(({ id, model: identifier }) => id !== model.id && identifier === model.model)
    ) {
      error.value = t("settings.ai.models.duplicateIdentifier");
      return;
    }
    admittedMutation = runMutation(async () => {
      await saveAiConnectionModel(connection.id, model);
      closeEditor();
      return true;
    });
    await admittedMutation;
  }

  async function removeModel(model: AiModelDefinition) {
    const connection = options.connection();
    if (busy.value || modelInUse(model.id)) return;
    if (
      !(await confirmAction({
        title: t("common.delete"),
        description: t("settings.ai.models.delete.title", { name: model.name }),
        confirmLabel: t("common.delete"),
        danger: true,
      }))
    )
      return;
    admittedMutation = runMutation(async () => {
      await removeAiConnectionModel(connection.id, model.id);
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
    selected,
    dirty,
    busy,
    error,
    editingModel,
    modelInUse,
    openModel,
    requestClose,
    saveModel,
    removeModel,
  };
}
