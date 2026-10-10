<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { connectorQuery, connectionQuery } from "../../../foundation/extensions/extension-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import ExtensionEmptyState from "../components/ExtensionEmptyState.vue";
import type { ExtensionDestination } from "../extension-destination";
import ConnectorConfiguration from "./ConnectorConfiguration.vue";
import ConnectorInspection from "./ConnectorInspection.vue";
import { useConnectorEditing } from "./use-connector-editing";
const props = defineProps<{ selectedId: string }>();
const emit = defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const connectors = useQuery(connectorQuery);
const connections = useQuery(connectionQuery);
const selected = computed(() =>
  connectors.data.value?.find((connector) => connector.id === props.selectedId),
);
const { changes, editing, draft, endpointFields, edit, closeEditor, save, deleteConnector } =
  useConnectorEditing();
async function close() {
  if (await closeEditor()) emit("navigate", { tab: "mcp", connectorId: props.selectedId });
}
async function commit() {
  if (await save()) emit("navigate", { tab: "mcp", connectorId: props.selectedId });
}
async function deleteService() {
  if (selected.value && (await deleteConnector(selected.value))) emit("navigate", { tab: "mcp" });
}
</script>
<template>
  <section class="max-w-xl space-y-4">
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <ConnectorConfiguration
      v-if="editing"
      v-model="draft"
      v-model:endpoint-fields="endpointFields"
      :busy="changes.isPending.value"
      :title="$t('extensions.connections.configure')"
      :error="changes.error.value ? getErrorMessage(changes.error.value) : undefined"
      @save="commit"
      @close="close"
    />
    <ConnectorInspection
      v-else-if="selected && connections.data.value"
      :connector="selected"
      :connections="connections.data.value"
      :busy="changes.isPending.value"
      @edit="edit(selected)"
      @delete="deleteService"
      @navigate="$emit('navigate', $event)"
    />
    <p
      v-else-if="connectors.isPending.value || connections.isPending.value"
      role="status"
      class="flex items-center gap-2 text-caption text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </p>
    <ExtensionEmptyState
      v-else-if="!connectors.error.value && !connections.error.value"
      :title="$t('extensions.overview.notFound')"
      :description="$t('extensions.overview.notFoundHint')"
    >
      <UiButton @click="$emit('navigate', { tab: 'mcp' })">{{ $t("extensions.overview.back") }}</UiButton>
    </ExtensionEmptyState>
    <p
      v-if="connectors.error.value || connections.error.value || (!editing && changes.error.value)"
      role="alert"
      class="text-caption text-fg-danger"
    >
      {{ getErrorMessage(connectors.error.value || connections.error.value || changes.error.value) }}
    </p>
  </section>
</template>
