<script setup lang="ts">
import { useMutation, useQuery } from "@tanstack/vue-query";
import { computed, ref, shallowRef, toRaw, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  createMcpConnection,
  updateMcpConnection,
  checkMcpConnection,
  deleteMcpConnection,
} from "../../../foundation/extensions/connection-operations";
import { connectionQuery, connectorQuery } from "../../../foundation/extensions/extension-query";
import type { ConnectorAuthorization } from "../../../shared/contracts/generated/ConnectorAuthorization";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpToolPermission } from "../../../shared/contracts/generated/McpToolPermission";
import UiButton from "../../../shared/ui/UiButton.vue";
import { useExtensionChanges } from "../use-extension-changes";
import { connectionDraft } from "./connection-draft";
import ConnectorAccounts from "./ConnectorAccounts.vue";
import McpCatalogBrowser from "./McpCatalogBrowser.vue";
import McpConnectionEditor from "./McpConnectionEditor.vue";
import McpToolSelection from "./McpToolSelection.vue";
const query = useQuery(connectionQuery);
const connectors = useQuery(connectorQuery);
const changes = useExtensionChanges();
const editing = shallowRef(false);
const draft = ref(connectionDraft());
const authorization = shallowRef<ConnectorAuthorization>({ type: "none" });
const selectedId = shallowRef("");
const selected = computed(() => query.data.value?.find((connection) => connection.id === selectedId.value));
const catalog = shallowRef<McpConnectionCatalog>();
const permissions = shallowRef<McpToolPermission[]>([]);
const checking = useMutation({
  mutationFn: (connection: McpConnection) => checkMcpConnection(connection.id),
  onSuccess: (value, connection) => {
    if (connection.id !== selectedId.value || connection.revision !== selected.value?.revision) return;
    catalog.value = value;
    permissions.value = selected.value?.tools.map((permission) => ({ ...permission })) ?? [];
  },
});
function edit(connection?: McpConnection) {
  draft.value = connection ? structuredClone(toRaw(connection)) : connectionDraft();
  authorization.value = connection
    ? structuredClone(
        toRaw(
          connectors.data.value!.find((connector) => connector.id === connection.connectorId)!.authorization,
        ),
      )
    : { type: "none" };
  editing.value = true;
}
function select(id: string) {
  selectedId.value = id;
}
watch(
  () => `${selected.value?.id ?? ""}:${selected.value?.revision ?? ""}`,
  () => {
    catalog.value = undefined;
    permissions.value = [];
    checking.reset();
  },
);
function mutate(operation: () => Promise<unknown>) {
  void changes.mutateAsync(operation).catch(() => {});
}
async function save() {
  await changes
    .mutateAsync(async () => {
      if (draft.value.id) await updateMcpConnection(draft.value, authorization.value);
      else await createMcpConnection(draft.value, authorization.value);
      editing.value = false;
    })
    .catch(() => {});
}
function savePermissions() {
  if (selected.value) {
    const connection = selected.value;
    mutate(() => updateMcpConnection({ ...connection, tools: permissions.value }));
  }
}
</script>

<template>
  <section class="space-y-4">
    <UiButton :disabled="changes.isPending.value" data-testid="create-mcp-connection" @click="edit()">{{
      $t("extensions.addConnection")
    }}</UiButton>
    <p class="text-caption text-fg-secondary">{{ $t("extensions.externalServiceBilling") }}</p>
    <p
      v-if="query.error.value || connectors.error.value || changes.error.value || checking.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{
        getErrorMessage(
          query.error.value || connectors.error.value || changes.error.value || checking.error.value,
        )
      }}
    </p>
    <McpConnectionEditor
      v-if="editing"
      v-model="draft"
      v-model:authorization="authorization"
      :busy="changes.isPending.value"
      @save="save"
      @close="editing = false"
    />
    <p v-if="query.isPending.value" class="text-caption text-fg-secondary">{{ $t("extensions.loading") }}</p>
    <p v-else-if="!query.error.value && !query.data.value?.length" class="text-caption text-fg-secondary">
      {{ $t("extensions.emptyConnections") }}
    </p>
    <section
      v-for="connection in query.data.value"
      :key="connection.id"
      class="space-y-2 border-b border-stroke-divider py-3"
      :data-testid="`mcp-connection-${connection.id}`"
    >
      <h2 class="text-body font-medium">{{ connection.name }}</h2>
      <p class="text-caption break-all text-fg-secondary">
        {{
          connection.endpoint.type === "streamable_http"
            ? connection.endpoint.url
            : connection.endpoint.executable
        }}
      </p>
      <div class="flex flex-wrap gap-2">
        <UiButton
          size="sm"
          :disabled="changes.isPending.value || !connectors.data.value"
          @click="edit(connection)"
          >{{ $t("extensions.configure") }}</UiButton
        >
        <UiButton
          size="sm"
          :disabled="changes.isPending.value"
          @click="mutate(() => updateMcpConnection({ ...connection, enabled: !connection.enabled }))"
          >{{ $t(connection.enabled ? "extensions.disable" : "extensions.enable") }}</UiButton
        >
        <UiButton size="sm" @click="select(connection.id)">{{ $t("extensions.accountsAndTools") }}</UiButton>
        <UiButton
          v-if="!connection.pluginId"
          size="sm"
          variant="dangerSubtle"
          :disabled="changes.isPending.value"
          @click="mutate(() => deleteMcpConnection(connection.id))"
          >{{ $t("extensions.delete") }}</UiButton
        >
      </div>
    </section>
    <template v-if="selected">
      <h2 class="text-body font-medium">{{ selected.name }}</h2>
      <ConnectorAccounts :connection="selected" />
      <UiButton
        :disabled="checking.isPending.value"
        data-testid="check-mcp-connection"
        @click="checking.mutate(selected)"
        >{{ $t("extensions.checkConnection") }}</UiButton
      >
      <McpCatalogBrowser v-if="catalog" :key="selected.revision" :connection="selected" :catalog="catalog" />
      <McpToolSelection
        v-if="catalog"
        v-model="permissions"
        :catalog="catalog"
        :busy="changes.isPending.value"
        @save="savePermissions"
      />
    </template>
  </section>
</template>
