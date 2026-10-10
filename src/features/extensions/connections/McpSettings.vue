<script setup lang="ts">
import { Plus, Server } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { connectionQuery, connectorQuery } from "../../../foundation/extensions/extension-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSettingsHeaderActions from "../../../shared/ui/UiSettingsHeaderActions.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionAvailability from "../components/ExtensionAvailability.vue";
import ExtensionEmptyState from "../components/ExtensionEmptyState.vue";
import ExtensionIdentity from "../components/ExtensionIdentity.vue";
import ConnectorServiceSettings from "../connectors/ConnectorServiceSettings.vue";
import type { ExtensionDestination } from "../extension-destination";
import McpConnectionEnablement from "./McpConnectionEnablement.vue";
import McpConnectionInspection from "./McpConnectionInspection.vue";
import McpServerCreation from "./McpServerCreation.vue";
const props = withDefaults(
  defineProps<{
    search?: string;
    selectedId?: string | undefined;
    connectorId?: string | undefined;
    action?: "create" | undefined;
  }>(),
  {
    search: "",
  },
);
defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const connections = useQuery(connectionQuery);
const connectors = useQuery(connectorQuery);
const connectorById = computed(
  () => new Map((connectors.data.value ?? []).map((connector) => [connector.id, connector])),
);
const visible = computed(() =>
  (connections.data.value ?? [])
    .filter((connection) => !connectorById.value.get(connection.connectorId)?.pluginId)
    .filter((connection) =>
      `${connection.name} ${connectorById.value.get(connection.connectorId)?.name ?? ""}`
        .toLowerCase()
        .includes(props.search.trim().toLowerCase()),
    ),
);
const selected = computed(() =>
  connections.data.value?.find((connection) => connection.id === props.selectedId),
);
const selectedConnector = computed(() =>
  selected.value ? connectorById.value.get(selected.value.connectorId) : undefined,
);
const inspecting = computed(() => Boolean(props.selectedId || props.connectorId || props.action));
const error = computed(() => connections.error.value || connectors.error.value);
</script>
<template>
  <section class="space-y-4">
    <UiSettingsHeaderActions v-if="!inspecting">
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
      v-else-if="connectorId"
      :key="connectorId"
      :selected-id="connectorId"
      @navigate="$emit('navigate', $event)"
    />
    <McpConnectionInspection
      v-else-if="selected && selectedConnector"
      :connection="selected"
      :connector="selectedConnector"
      @navigate="$emit('navigate', $event)"
    />
    <div
      v-else-if="connections.isPending.value || connectors.isPending.value"
      role="status"
      class="flex justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="selectedId && !error"
      :title="$t('extensions.overview.notFound')"
      :description="$t('extensions.overview.notFoundHint')"
      ><UiButton @click="$emit('navigate', { tab: 'mcp' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <template v-else-if="!inspecting">
      <p class="text-caption text-fg-secondary">{{ $t("extensions.connections.description") }}</p>
      <ExtensionEmptyState
        v-if="!error && !visible.length"
        :title="$t(search ? 'extensions.plugins.search.empty' : 'extensions.connections.emptyServers')"
        :description="$t(search ? 'extensions.plugins.search.hint' : 'extensions.connections.emptyHint')"
      />
      <UiSettingsSection v-if="visible.length">
        <div class="divide-y divide-stroke-divider px-4">
          <ExtensionIdentity
            v-for="connection in visible"
            :key="connection.id"
            :name="connection.name"
            :description="connectorById.get(connection.connectorId)?.name"
            :icon="Server"
            :muted="!connection.enabled"
            :data-testid="`mcp-server-${connection.id}`"
            @inspect="$emit('navigate', { tab: 'mcp', id: connection.id })"
          >
            <template #status
              ><ExtensionAvailability
                class="mt-1"
                kind="connections"
                :extension-id="connection.id"
                :enabled="connection.enabled"
            /></template>
            <UiButton
              size="sm"
              variant="ghost"
              @click="$emit('navigate', { tab: 'mcp', id: connection.id })"
              >{{ $t("extensions.connections.configure") }}</UiButton
            >
            <McpConnectionEnablement
              :connection="connection"
              :configured="Boolean(connectorById.get(connection.connectorId)?.endpoint)"
            />
          </ExtensionIdentity>
        </div>
      </UiSettingsSection>
    </template>
  </section>
</template>
