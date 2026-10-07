<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import {
  createMcpConnection,
  updateMcpConnection,
  deleteMcpConnection,
} from "../../../foundation/extensions/connection-operations";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";
import McpConnectionSettings from "../connections/McpConnectionSettings.vue";
import { useExtensionChanges } from "../use-extension-changes";

const props = defineProps<{ connector: ConnectorDefinition; connections: McpConnection[]; busy: boolean }>();
const { t } = useI18n();
const adding = shallowRef(false);
const changes = useExtensionChanges();
const name = shallowRef("");
const selectedId = shallowRef("");
const selected = computed(() => props.connections.find((connection) => connection.id === selectedId.value));
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
    adding.value = false;
  });
}
async function remove(connection: McpConnection) {
  if (
    await confirmAction({
      title: t("extensions.deleteConnectionNamed", { name: connection.name }),
      description: t("extensions.deleteConnectionDescription"),
      confirmLabel: t("extensions.delete"),
      danger: true,
    })
  )
    mutate(() => deleteMcpConnection(connection));
}
</script>
<template>
  <div class="space-y-4">
    <section class="space-y-3">
      <header class="flex items-center justify-between gap-3">
        <h4 class="text-body font-medium">{{ $t("extensions.connections") }}</h4>
        <UiButton
          size="sm"
          variant="ghost"
          :disabled="busy || changes.isPending.value || !connector.endpoint"
          :aria-expanded="adding"
          data-testid="add-connection-instance"
          @click="adding = !adding"
          ><Plus class="size-3.5" />{{ $t("extensions.addConnection") }}</UiButton
        >
      </header>
      <form v-if="adding" class="space-y-2 rounded-lg bg-surface-muted p-3" @submit.prevent="create">
        <label class="block space-y-1.5 text-caption text-fg-secondary"
          >{{ $t("extensions.connectionName")
          }}<UiInput
            v-model="name"
            :disabled="busy || changes.isPending.value"
            data-testid="connection-instance-name"
        /></label>
        <div class="flex justify-end gap-2">
          <UiButton size="sm" variant="ghost" :disabled="changes.isPending.value" @click="adding = false">{{
            $t("extensions.cancel")
          }}</UiButton
          ><UiButton
            type="submit"
            variant="primary"
            size="sm"
            :disabled="!name.trim() || busy || changes.isPending.value"
            data-testid="create-connection-instance"
            >{{ $t("extensions.addConnection") }}</UiButton
          >
        </div>
      </form>
      <p v-if="!connections.length" class="rounded-md bg-surface-muted p-3 text-caption text-fg-secondary">
        {{ $t("extensions.emptyConnections") }}
      </p>
      <div
        v-for="connection in connections"
        :key="connection.id"
        class="flex flex-wrap items-center gap-2 rounded-md bg-surface-muted px-3 py-2.5"
        :data-testid="`mcp-connection-${connection.id}`"
      >
        <span class="min-w-0 flex-1 truncate text-body font-medium">{{ connection.name }}</span>
        <UiSwitch
          :model-value="connection.enabled"
          :disabled="busy || changes.isPending.value"
          :label="
            $t(connection.enabled ? 'extensions.disableNamed' : 'extensions.enableNamed', {
              name: connection.name,
            })
          "
          @update:model-value="
            mutate(() => updateMcpConnection({ ...connection, enabled: !connection.enabled }))
          "
        />
        <UiButton
          size="sm"
          variant="ghost"
          :aria-expanded="selectedId === connection.id"
          @click="selectedId = selectedId === connection.id ? '' : connection.id"
          >{{ $t("extensions.accountsAndTools") }}</UiButton
        >
        <UiButton
          size="sm"
          variant="ghost"
          class="text-fg-danger"
          :disabled="busy || changes.isPending.value"
          :data-testid="`delete-connection-${connection.id}`"
          @click="remove(connection)"
          >{{ $t("extensions.delete") }}</UiButton
        >
      </div>
    </section>
    <p v-if="changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(changes.error.value) }}
    </p>
    <section v-if="selected" class="space-y-3 rounded-lg bg-surface-muted p-3">
      <h4 class="text-body font-medium">{{ selected.name }} · {{ $t("extensions.accountsAndTools") }}</h4>
      <McpConnectionSettings :key="selected.id" :connection="selected" :connector="connector" />
    </section>
  </div>
</template>
