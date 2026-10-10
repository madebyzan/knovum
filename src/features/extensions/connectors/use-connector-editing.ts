import { cloneDeep, isEqual } from "es-toolkit";
import { computed, onScopeDispose, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { createMcpServer } from "../../../foundation/extensions/connection-operations";
import {
  deleteConnectorDefinition,
  updateConnectorDefinition,
} from "../../../foundation/extensions/connector-operations";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { ExtensionChange } from "../../../shared/contracts/generated/ExtensionChange";
import { useSettingsLeaveContext } from "../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { buildEndpointFields } from "../connections/endpoint-fields";
import { useExtensionChanges } from "../use-extension-changes";
import { connectorDraft } from "./connector-draft";
export function useConnectorEditing() {
  const { t } = useI18n();
  const changes = useExtensionChanges();
  const editing = shallowRef(false);
  const draft = ref(connectorDraft());
  const baseline = ref(connectorDraft());
  const endpointFields = ref(buildEndpointFields(draft.value.endpoint));
  const baselineEndpointFields = ref(cloneDeep(endpointFields.value));
  const dirty = computed(
    () =>
      editing.value &&
      (!isEqual(draft.value, baseline.value) || !isEqual(endpointFields.value, baselineEndpointFields.value)),
  );
  watch(
    () => draft.value.endpoint.type,
    () => {
      endpointFields.value = buildEndpointFields(draft.value.endpoint);
    },
  );
  let pendingMutation: Promise<boolean> | undefined;

  function edit(connector?: ConnectorDefinition) {
    const editable = connector ? cloneDeep(connector) : connectorDraft();
    draft.value = { ...editable, endpoint: editable.endpoint! };
    baseline.value = cloneDeep(draft.value);
    endpointFields.value = buildEndpointFields(draft.value.endpoint);
    baselineEndpointFields.value = cloneDeep(endpointFields.value);
    editing.value = true;
  }

  const leaveParticipant = {
    async prepare() {
      if (pendingMutation && !(await pendingMutation)) return false;
      return !dirty.value || confirmDiscard();
    },
    finalize() {
      editing.value = false;
    },
  };
  onScopeDispose(useSettingsLeaveContext().register(leaveParticipant));

  async function closeEditor() {
    if (!(await leaveParticipant.prepare())) return false;
    leaveParticipant.finalize();
    return true;
  }

  function save() {
    pendingMutation = trackMutation(async () => {
      const change = draft.value.id
        ? await updateConnectorDefinition(draft.value)
        : await createMcpServer(draft.value);
      editing.value = false;
      return change;
    });
    return pendingMutation;
  }

  async function deleteConnector(connector: ConnectorDefinition) {
    if (
      await confirmAction({
        title: t("extensions.connectors.delete.title", { name: connector.name }),
        description: t("extensions.connectors.delete.description"),
        confirmLabel: t("common.delete"),
        danger: true,
      })
    )
      return (pendingMutation = trackMutation(() => deleteConnectorDefinition(connector)));
    return false;
  }

  function trackMutation(mutation: () => Promise<ExtensionChange<unknown>>) {
    const completion: Promise<boolean> = changes
      .mutateAsync(mutation)
      .then(
        () => true,
        () => false,
      )
      .finally(() => {
        if (pendingMutation === completion) pendingMutation = undefined;
      });
    return completion;
  }

  function confirmDiscard() {
    return confirmAction({
      title: t("extensions.connectors.discard.title"),
      description: t("extensions.connectors.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    });
  }
  return { changes, editing, draft, endpointFields, edit, closeEditor, save, deleteConnector };
}
