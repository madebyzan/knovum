<script setup lang="ts">
import { Plus, Server } from "@lucide/vue";
import { computed } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSettingsHeaderActions from "../../../../shared/ui/UiSettingsHeaderActions.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import ExtensionAvailability from "../../components/ExtensionAvailability.vue";
import ExtensionEmptyState from "../../components/ExtensionEmptyState.vue";
import ExtensionIdentity from "../../components/ExtensionIdentity.vue";
import { isAppConnector } from "../../connectors/app-connector";
import ConnectorServiceSettings from "../../connectors/components/ConnectorServiceSettings.vue";
import { useExtensionInventory } from "../../connectors/use-extension-inventory";
import type { ExtensionDestination } from "../../navigation/extension-destination";
import McpConnectionEnablement from "./McpConnectionEnablement.vue";
import McpConnectionInspection from "./McpConnectionInspection.vue";
import McpServerCreation from "./McpServerCreation.vue";
const props = withDefaults(
  defineProps<{
    search?: string;
    connectionId?: string | undefined;
    connectorId?: string | undefined;
    action?: "create" | undefined;
  }>(),
  {
    search: "",
  },
);
defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const { connectors, connections, pluginById, error, isLoading } = useExtensionInventory();
const connectorById = computed(
  () =>
    new Map(
      (connectors.data.value ?? [])
        .filter(
          (connector) =>
            !connector.pluginId ||
            (pluginById.value.has(connector.pluginId) &&
              !isAppConnector(connector, pluginById.value.get(connector.pluginId))),
        )
        .map((connector) => [connector.id, connector]),
    ),
);
const visibleConnections = computed(() =>
  (connections.data.value ?? [])
    .filter((connection) => connectorById.value.has(connection.connectorId))
    .filter((connection) =>
      `${connection.name} ${connectorById.value.get(connection.connectorId)?.name ?? ""}`
        .toLowerCase()
        .includes(props.search.trim().toLowerCase()),
    ),
);
const selectedConnection = computed(() =>
  connections.data.value?.find(
    (connection) => connection.id === props.connectionId && connectorById.value.has(connection.connectorId),
  ),
);
const selectedConnector = computed(() =>
  selectedConnection.value ? connectorById.value.get(selectedConnection.value.connectorId) : undefined,
);
const isBrowsing = computed(() => !(props.connectionId || props.connectorId || props.action));
function isConnectionEnabled(connectorId: string, enabled: boolean) {
  const pluginId = connectorById.value.get(connectorId)?.pluginId;
  return pluginId ? Boolean(pluginById.value.get(pluginId)?.enabled) : enabled;
}
</script>
<template>
  <section class="space-y-4">
    <UiSettingsHeaderActions v-if="isBrowsing">
      <UiButton
        size="sm"
        data-testid="add-mcp-server"
        @click="$emit('navigate', { tab: 'mcp', action: 'create' })"
        ><Plus class="size-4" />{{ $t("extensions.connections.addServer") }}</UiButton
      >
    </UiSettingsHeaderActions>
    <p v-if="error" role="alert" class="text-body text-fg-danger">{{ getErrorMessage(error) }}</p>
    <McpServerCreation v-if="action === 'create'" @navigate="$emit('navigate', $event)" />
    <ConnectorServiceSettings
      v-else-if="connectorId && connectorById.has(connectorId)"
      :key="connectorId"
      :connector-id="connectorId"
      @navigate="$emit('navigate', $event)"
    />
    <McpConnectionInspection
      v-else-if="selectedConnection && selectedConnector"
      :connection="selectedConnection"
      :connector="selectedConnector"
      @navigate="$emit('navigate', $event)"
    />
    <div
      v-else-if="isLoading"
      role="status"
      class="flex justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="(connectionId || connectorId) && !error"
      :title="$t('extensions.overview.notFound.title')"
      :description="$t('extensions.overview.notFound.description')"
      ><UiButton @click="$emit('navigate', { tab: 'mcp' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <template v-else-if="isBrowsing">
      <p class="text-caption text-fg-secondary">{{ $t("extensions.connections.description") }}</p>
      <ExtensionEmptyState
        v-if="!error && !visibleConnections.length"
        :title="$t(search ? 'common.search.empty' : 'extensions.connections.emptyServers.title')"
        :description="$t(search ? 'common.search.hint' : 'extensions.connections.emptyServers.description')"
      />
      <UiSettingsSection v-if="visibleConnections.length">
        <div class="divide-y divide-stroke-divider px-4">
          <ExtensionIdentity
            v-for="connection in visibleConnections"
            :key="connection.id"
            :name="connection.name"
            :description="connectorById.get(connection.connectorId)?.name"
            :icon="Server"
            :muted="!isConnectionEnabled(connection.connectorId, connection.enabled)"
            :data-testid="`mcp-server-${connection.id}`"
            @inspect="$emit('navigate', { tab: 'mcp', id: connection.id })"
          >
            <template #status
              ><ExtensionAvailability
                class="mt-1"
                kind="connections"
                :extension-id="connection.id"
                :enabled="isConnectionEnabled(connection.connectorId, connection.enabled)"
                :plugin-id="connectorById.get(connection.connectorId)?.pluginId"
            /></template>
            <UiButton
              size="sm"
              variant="ghost"
              @click="$emit('navigate', { tab: 'mcp', id: connection.id })"
              >{{ $t("extensions.connections.configure") }}</UiButton
            >
            <McpConnectionEnablement
              v-if="!connectorById.get(connection.connectorId)?.pluginId"
              :connection="connection"
              :configured="Boolean(connectorById.get(connection.connectorId)?.endpoint)"
            />
          </ExtensionIdentity>
        </div>
      </UiSettingsSection>
    </template>
  </section>
</template>
