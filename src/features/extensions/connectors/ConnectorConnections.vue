<script setup lang="ts">
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  createMcpConnection,
  updateMcpConnection,
  deleteMcpConnection,
} from "../../../foundation/extensions/connection-operations";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import McpConnectionSettings from "../connections/McpConnectionSettings.vue";
import { useExtensionChanges } from "../use-extension-changes";
import ConnectorAccounts from "./ConnectorAccounts.vue";
import PluginConnectorConfiguration from "./PluginConnectorConfiguration.vue";

const props = defineProps<{ connector: ConnectorDefinition; connections: McpConnection[]; busy: boolean }>();
defineEmits<{ edit: []; remove: [] }>();
const changes = useExtensionChanges();
const name = shallowRef("");
const selectedId = shallowRef("");
const connections = computed(() =>
  props.connections.filter((connection) => connection.connectorId === props.connector.id),
);
const selected = computed(() => connections.value.find((connection) => connection.id === selectedId.value));
function mutate(operation: () => Promise<unknown>) {
  void changes
    .mutateAsync(async () => {
      await operation();
    })
    .catch(() => {});
}
function create() {
  mutate(async () => {
    const connection = await createMcpConnection(props.connector.id, name.value);
    selectedId.value = connection.id;
    name.value = "";
  });
}
</script>

<template>
  <section
    class="space-y-3 rounded-sm border border-stroke-divider p-3"
    :data-testid="`connector-${connector.id}`"
  >
    <header class="flex items-start justify-between gap-3">
      <div class="min-w-0">
        <h2 class="text-body font-medium">{{ connector.name }}</h2>
        <p v-if="connector.description" class="mt-1 text-caption text-fg-secondary">
          {{ connector.description }}
        </p>
        <p class="mt-1 text-caption text-fg-tertiary">
          {{ $t(connector.pluginId ? "extensions.pluginConnector" : "extensions.customConnector") }} · MCP
        </p>
      </div>
      <div v-if="!connector.pluginId" class="flex gap-2">
        <UiButton size="sm" :disabled="busy" data-testid="edit-connector" @click="$emit('edit')">{{
          $t("extensions.configure")
        }}</UiButton>
        <UiButton
          size="sm"
          variant="dangerSubtle"
          :disabled="busy"
          data-testid="delete-connector"
          @click="$emit('remove')"
          >{{ $t("extensions.delete") }}</UiButton
        >
      </div>
    </header>
    <PluginConnectorConfiguration v-if="connector.pluginId" :connector="connector" />
    <ConnectorAccounts
      v-if="connector.endpoint && connector.authorization.type !== 'none'"
      :connector="connector"
    />
    <form class="flex items-end gap-2" @submit.prevent="create">
      <label class="min-w-0 flex-1 text-caption"
        >{{ $t("extensions.connectionName") }}<UiInput v-model="name" data-testid="connection-instance-name"
      /></label>
      <UiButton
        type="submit"
        size="sm"
        :disabled="!name.trim() || changes.isPending.value"
        data-testid="create-connection-instance"
        >{{ $t("extensions.addConnection") }}</UiButton
      >
    </form>
    <p v-if="changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(changes.error.value) }}
    </p>
    <p v-if="!connections.length" class="text-caption text-fg-secondary">
      {{ $t("extensions.emptyConnections") }}
    </p>
    <div
      v-for="connection in connections"
      :key="connection.id"
      class="flex items-center gap-2"
      :data-testid="`mcp-connection-${connection.id}`"
    >
      <span class="min-w-0 flex-1 truncate text-body">{{ connection.name }}</span>
      <UiButton
        size="sm"
        :disabled="changes.isPending.value"
        @click="mutate(() => updateMcpConnection({ ...connection, enabled: !connection.enabled }))"
        >{{ $t(connection.enabled ? "extensions.disable" : "extensions.enable") }}</UiButton
      >
      <UiButton size="sm" @click="selectedId = connection.id">{{
        $t("extensions.accountsAndTools")
      }}</UiButton>
      <UiButton
        size="sm"
        variant="dangerSubtle"
        :disabled="changes.isPending.value"
        @click="mutate(() => deleteMcpConnection(connection))"
        >{{ $t("extensions.delete") }}</UiButton
      >
    </div>
    <McpConnectionSettings v-if="selected" :key="selected.id" :connection="selected" :connector="connector" />
  </section>
</template>
