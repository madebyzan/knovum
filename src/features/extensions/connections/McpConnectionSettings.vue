<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query";
import { shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  updateMcpConnection,
  checkMcpConnection,
} from "../../../foundation/extensions/connection-operations";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../../shared/contracts/generated/McpConnectionCatalog";
import type { McpToolPermission } from "../../../shared/contracts/generated/McpToolPermission";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useExtensionChanges } from "../use-extension-changes";
import ConnectionAccountSelection from "./ConnectionAccountSelection.vue";
import McpCatalogBrowser from "./McpCatalogBrowser.vue";
import McpToolSelection from "./McpToolSelection.vue";

const props = defineProps<{ connection: McpConnection; connector: ConnectorDefinition }>();
const changes = useExtensionChanges();
const name = shallowRef("");
const catalog = shallowRef<McpConnectionCatalog>();
const permissions = shallowRef<McpToolPermission[]>([]);
const checking = useMutation({
  mutationFn: (connection: McpConnection) => checkMcpConnection(connection.id),
  onSuccess: (value, connection) => {
    if (connection.id !== props.connection.id || connection.revision !== props.connection.revision) return;
    catalog.value = value;
    permissions.value = props.connection.tools.map((permission) => ({ ...permission }));
  },
});
watch(
  () => props.connection.revision,
  () => {
    name.value = props.connection.name;
    catalog.value = undefined;
    permissions.value = [];
    checking.reset();
  },
  { immediate: true },
);
function save(update: Partial<Pick<McpConnection, "name" | "tools">>) {
  void changes.mutateAsync(() => updateMcpConnection({ ...props.connection, ...update })).catch(() => {});
}
</script>

<template>
  <section class="space-y-3 border-t border-stroke-divider pt-3">
    <form class="flex items-end gap-2" @submit.prevent="save({ name })">
      <label class="min-w-0 flex-1 text-caption"
        >{{ $t("extensions.connectionName") }}<UiInput v-model="name"
      /></label>
      <UiButton
        type="submit"
        size="sm"
        :disabled="!name.trim() || name === connection.name || changes.isPending.value"
        >{{ $t("extensions.save") }}</UiButton
      >
    </form>
    <ConnectionAccountSelection v-if="connector.authorization.type !== 'none'" :connection="connection" />
    <UiButton
      :disabled="checking.isPending.value"
      data-testid="check-mcp-connection"
      @click="checking.mutate(connection)"
      >{{ $t("extensions.checkConnection") }}</UiButton
    >
    <p v-if="changes.error.value || checking.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(changes.error.value || checking.error.value) }}
    </p>
    <McpCatalogBrowser
      v-if="catalog"
      :key="connection.revision"
      :connection="connection"
      :catalog="catalog"
    />
    <McpToolSelection
      v-if="catalog"
      v-model="permissions"
      :catalog="catalog"
      :busy="changes.isPending.value"
      @save="save({ tools: permissions })"
    />
  </section>
</template>
