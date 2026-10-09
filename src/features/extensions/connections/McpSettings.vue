<script setup lang="ts">
import { Plus, Server } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { connectionQuery, connectorQuery } from "../../../foundation/extensions/extension-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionAvailability from "../components/ExtensionAvailability.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import ExtensionEmptyState from "../components/ExtensionEmptyState.vue";
import ExtensionIdentity from "../components/ExtensionIdentity.vue";
import ConnectorConfiguration from "../connectors/ConnectorConfiguration.vue";
import { useConnectorEditing } from "../connectors/use-connector-editing";
import type { ExtensionDestination } from "../extension-destination";
import McpConnectionEnablement from "./McpConnectionEnablement.vue";
import McpConnectionInspection from "./McpConnectionInspection.vue";
const props = withDefaults(defineProps<{ search?: string; selectedId?: string | undefined }>(), {
  search: "",
});
defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const connections = useQuery(connectionQuery);
const connectors = useQuery(connectorQuery);
const connectorById = computed(
  () => new Map((connectors.data.value ?? []).map((connector) => [connector.id, connector])),
);
const visible = computed(() =>
  (connections.data.value ?? []).filter((connection) =>
    `${connection.name} ${connectorById.value.get(connection.connectorId)?.name ?? ""}`
      .toLowerCase()
      .includes(props.search.trim().toLowerCase()),
  ),
);
const standalone = computed(() =>
  visible.value.filter((connection) => !connectorById.value.get(connection.connectorId)?.pluginId),
);
const packaged = computed(() =>
  visible.value.filter((connection) => connectorById.value.get(connection.connectorId)?.pluginId),
);
const populatedGroups = computed(() =>
  [
    { name: "standalone", connections: standalone.value },
    { name: "packaged", connections: packaged.value },
  ].filter((group) => group.connections.length),
);
const selected = computed(() =>
  connections.data.value?.find((connection) => connection.id === props.selectedId),
);
const selectedConnector = computed(() =>
  selected.value ? connectorById.value.get(selected.value.connectorId) : undefined,
);
const { changes, editing, draft, publicFields, edit, closeEditor, save } = useConnectorEditing({
  creation: "server",
});
const error = computed(() => connections.error.value || connectors.error.value || changes.error.value);
</script>
<template>
  <section class="space-y-4">
    <header v-if="!selectedId" class="flex justify-end">
      <UiButton :disabled="changes.isPending.value" data-testid="add-mcp-server" @click="edit()"
        ><Plus class="size-4" />{{ $t("extensions.connections.addServer") }}</UiButton
      >
    </header>
    <p v-if="error" role="alert" class="text-body text-fg-danger">{{ getErrorMessage(error) }}</p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <McpConnectionInspection
      v-if="selected && selectedConnector"
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
    <template v-else-if="!selectedId">
      <p class="text-caption text-fg-secondary">{{ $t("extensions.connections.description") }}</p>
      <ExtensionEmptyState
        v-if="!error && !visible.length"
        :title="$t(search ? 'extensions.plugins.search.empty' : 'extensions.connections.emptyServers')"
        :description="$t(search ? 'extensions.plugins.search.hint' : 'extensions.connections.emptyHint')"
      />
      <section v-for="group in populatedGroups" :key="group.name" class="space-y-2">
        <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
          {{ $t(`extensions.connections.${group.name}`) }}
        </h3>
        <ExtensionIdentity
          v-for="connection in group.connections"
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
              :plugin-id="connectorById.get(connection.connectorId)?.pluginId"
          /></template>
          <UiButton size="sm" variant="ghost" @click="$emit('navigate', { tab: 'mcp', id: connection.id })">{{
            $t("extensions.connections.configure")
          }}</UiButton>
          <McpConnectionEnablement
            :connection="connection"
            :configured="Boolean(connectorById.get(connection.connectorId)?.endpoint)"
          />
        </ExtensionIdentity>
      </section>
    </template>
    <ConnectorConfiguration
      v-model="draft"
      v-model:public-fields="publicFields"
      :open="editing"
      :busy="changes.isPending.value"
      :title="$t('extensions.connections.addServer')"
      :error="changes.error.value ? getErrorMessage(changes.error.value) : undefined"
      @save="save"
      @close="closeEditor"
    />
  </section>
</template>
