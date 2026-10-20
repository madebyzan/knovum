import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import {
  removeAiConnectionModel,
  saveAiConnectionModel,
} from "../../../../foundation/ai/connection-settings";
import { useSettingsEditorLeave } from "../../../../foundation/lifecycle/use-settings-editor-leave";
import type { AiConnectionSettings } from "../../../../shared/contracts/generated/AiConnectionSettings";
import type { AiModelDefinition } from "../../../../shared/contracts/generated/AiModelDefinition";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import { useAiEditorMutation } from "../use-ai-editor-mutation";

interface ModelEditingOptions {
  connection: () => AiConnectionSettings;
  protectedModelTargets: () => readonly string[];
}

export function useAiModelEditing(options: ModelEditingOptions) {
  const { t } = useI18n();
  const selected = shallowRef<string | null>();
  const dirty = shallowRef(false);
  const mutation = useAiEditorMutation();
  const { busy, error } = mutation;
  const editingModel = computed(() => options.connection().models.find(({ id }) => id === selected.value));

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

  const requestClose = useSettingsEditorLeave({
    pending: mutation.pending,
    dirty: () => dirty.value,
    confirmDiscard,
    close: closeEditor,
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
    await mutation.run(async () => {
      await saveAiConnectionModel(connection.id, model);
      closeEditor();
      return true;
    });
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
    await mutation.run(async () => {
      await removeAiConnectionModel(connection.id, model.id);
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
