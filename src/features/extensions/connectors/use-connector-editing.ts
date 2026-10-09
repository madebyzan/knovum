import { cloneDeep, isEqual } from "es-toolkit";
import { computed, onScopeDispose, ref, shallowRef } from "vue";
import type { ComposerTranslation } from "vue-i18n";

import {
  createConnectorDefinition,
  deleteConnectorDefinition,
  updateConnectorDefinition,
} from "../../../foundation/extensions/connector-operations";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { ExtensionChange } from "../../../shared/contracts/generated/ExtensionChange";
import { useSettingsLeaveContext } from "../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { useExtensionChanges } from "../use-extension-changes";
import { connectorDraft } from "./connector-draft";

export function useConnectorEditing(t: ComposerTranslation) {
  const changes = useExtensionChanges();
  const editing = shallowRef(false);
  const draft = ref(connectorDraft());
  const baseline = ref(connectorDraft());
  const dirty = computed(() => editing.value && !isEqual(draft.value, baseline.value));
  let admittedMutation: Promise<boolean> | undefined;

  function edit(connector?: ConnectorDefinition) {
    const editable = connector ? cloneDeep(connector) : connectorDraft();
    draft.value = { ...editable, endpoint: editable.endpoint! };
    baseline.value = cloneDeep(draft.value);
    editing.value = true;
  }

  async function closeEditor() {
    if (admittedMutation) {
      const succeeded = await admittedMutation;
      if (!succeeded) return false;
      if (!editing.value) return true;
    }
    if (dirty.value && !(await confirmDiscard(t))) return false;
    editing.value = false;
    return true;
  }

  const unregister = useSettingsLeaveContext().register({
    async prepare() {
      if (admittedMutation && !(await admittedMutation)) return false;
      return !dirty.value || confirmDiscard(t);
    },
    finalize() {
      editing.value = false;
    },
  });
  onScopeDispose(unregister);

  function save() {
    admittedMutation = trackMutation(async () => {
      const change = draft.value.id
        ? await updateConnectorDefinition(draft.value)
        : await createConnectorDefinition(draft.value);
      editing.value = false;
      return change;
    });
  }

  async function remove(connector: ConnectorDefinition) {
    if (
      await confirmAction({
        title: t("extensions.connectors.delete.title", { name: connector.name }),
        description: t("extensions.connectors.delete.description"),
        confirmLabel: t("common.delete"),
        danger: true,
      })
    )
      admittedMutation = trackMutation(() => deleteConnectorDefinition(connector));
  }

  function trackMutation(mutation: () => Promise<ExtensionChange<unknown>>) {
    const admitted: Promise<boolean> = changes
      .mutateAsync(mutation)
      .then(
        () => true,
        () => false,
      )
      .finally(() => {
        if (admittedMutation === admitted) admittedMutation = undefined;
      });
    return admitted;
  }

  return { changes, editing, draft, edit, closeEditor, save, remove };
}

function confirmDiscard(t: ComposerTranslation) {
  return confirmAction({
    title: t("extensions.connectors.discard.title"),
    description: t("extensions.connectors.discard.description"),
    confirmLabel: t("common.discardChanges"),
    danger: true,
  });
}
