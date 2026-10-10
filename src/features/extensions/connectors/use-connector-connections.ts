import { computed, onScopeDispose, provide, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import {
  createMcpConnection,
  deleteMcpConnection,
} from "../../../foundation/extensions/connection-operations";
import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
  useSettingsLeaveContext,
} from "../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { useExtensionChanges } from "../use-extension-changes";

interface ConnectorConnectionsOptions {
  connector: () => ConnectorDefinition;
  connections: () => McpConnection[];
}

export function useConnectorConnections(options: ConnectorConnectionsOptions) {
  const { t } = useI18n();
  const adding = shallowRef(false);
  const changes = useExtensionChanges();
  const name = shallowRef("");
  const selectedId = shallowRef("");
  const selected = computed(() => options.connections().find(({ id }) => id === selectedId.value));
  const outerLeaveCoordinator = useSettingsLeaveContext();
  const childLeaveParticipants = new Set<SettingsLeaveParticipant>();
  let preparedChildren: SettingsLeaveParticipant[] = [];
  let pendingMutation: Promise<boolean> | undefined;

  async function prepareChildLeave() {
    preparedChildren = [...childLeaveParticipants];
    for (const participant of preparedChildren) {
      // oxlint-disable-next-line no-await-in-loop -- nested editor prompts must resolve in order.
      if (!(await participant.prepare())) {
        preparedChildren = [];
        return false;
      }
    }
    return true;
  }
  function finalizeChildLeave() {
    for (const participant of preparedChildren) participant.finalize?.();
    preparedChildren = [];
  }
  async function allowChildLeave() {
    if (!(await prepareChildLeave())) return false;
    finalizeChildLeave();
    return true;
  }
  provide(settingsLeaveCoordinatorKey, {
    register(participant) {
      childLeaveParticipants.add(participant);
      return () => childLeaveParticipants.delete(participant);
    },
    async requestLeave() {
      if (!(await allowChildLeave())) throw new ApplicationLeaveCancelled();
    },
  });

  async function prepareLeave() {
    if (pendingMutation && !(await pendingMutation)) return false;
    if (adding.value && name.value.trim() && !(await confirmDiscard())) return false;
    return prepareChildLeave();
  }
  const unregister = outerLeaveCoordinator.register({
    prepare: prepareLeave,
    finalize() {
      adding.value = false;
      name.value = "";
      finalizeChildLeave();
    },
  });
  onScopeDispose(unregister);

  function mutate(operation: () => Promise<unknown>) {
    const completion: Promise<boolean> = changes
      .mutateAsync(async () => {
        await operation();
      })
      .then(
        () => true,
        () => false,
      )
      .finally(() => {
        if (pendingMutation === completion) pendingMutation = undefined;
      });
    pendingMutation = completion;
    return completion;
  }

  async function create() {
    if (!(await allowChildLeave())) return;
    await mutate(async () => {
      const connection = await createMcpConnection(options.connector().id, name.value);
      selectedId.value = connection.id;
      name.value = "";
      adding.value = false;
    });
  }

  async function toggleAdding() {
    if (adding.value && name.value.trim() && !(await confirmDiscard())) return;
    adding.value = !adding.value;
    if (!adding.value) name.value = "";
  }

  async function toggleSelected(connectionId: string) {
    if (!(await allowChildLeave())) return;
    selectedId.value = selectedId.value === connectionId ? "" : connectionId;
  }

  async function deleteConnection(connection: McpConnection) {
    if (
      !(await confirmAction({
        title: t("extensions.connections.delete.title", { name: connection.name }),
        description: t("extensions.connections.delete.description"),
        confirmLabel: t("common.delete"),
        danger: true,
      }))
    )
      return;
    await mutate(() => deleteMcpConnection(connection));
  }

  function confirmDiscard() {
    return confirmAction({
      title: t("extensions.connectors.discard.title"),
      description: t("extensions.connectors.discard.description"),
      confirmLabel: t("common.discardChanges"),
      danger: true,
    });
  }

  return {
    adding,
    changes,
    name,
    selectedId,
    selected,
    create,
    toggleAdding,
    toggleSelected,
    deleteConnection,
  };
}
