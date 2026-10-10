<script setup lang="ts">
import { Plus } from "@lucide/vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import ExtensionAvailability from "../../components/ExtensionAvailability.vue";
import McpConnectionEnablement from "../../connections/components/McpConnectionEnablement.vue";
import McpConnectionSettings from "../../connections/components/McpConnectionSettings.vue";
import { useConnectorConnections } from "../use-connector-connections";

const props = defineProps<{ connector: ConnectorDefinition; connections: McpConnection[]; busy: boolean }>();
const {
  adding,
  changes,
  name,
  selectedConnectionId,
  selected,
  create,
  toggleAdding,
  toggleSelected,
  deleteConnection,
} = useConnectorConnections({ connector: () => props.connector, connections: () => props.connections });
</script>
<template>
  <div class="space-y-4">
    <section class="space-y-3">
      <header class="flex items-center justify-between gap-3">
        <h4 class="text-body font-medium">{{ $t("extensions.connections.title") }}</h4>
        <UiButton
          v-if="!connector.pluginId"
          size="sm"
          variant="ghost"
          :disabled="busy || changes.isPending.value || !connector.endpoint"
          :aria-expanded="adding"
          data-testid="add-mcp-connection"
          @click="toggleAdding"
          ><Plus class="size-3.5" />{{ $t("extensions.connections.add") }}</UiButton
        >
      </header>
      <form v-if="adding" class="space-y-2 rounded-lg bg-surface-muted p-3" @submit.prevent="create">
        <label class="block space-y-1.5 text-caption text-fg-secondary"
          >{{ $t("extensions.connections.name")
          }}<UiInput
            v-model="name"
            :disabled="busy || changes.isPending.value"
            data-testid="mcp-connection-name"
        /></label>
        <div class="flex justify-end gap-2">
          <UiButton size="sm" variant="ghost" :disabled="changes.isPending.value" @click="toggleAdding">{{
            $t("common.cancel")
          }}</UiButton
          ><UiButton
            type="submit"
            variant="primary"
            size="sm"
            :disabled="!name.trim() || busy || changes.isPending.value"
            data-testid="create-mcp-connection"
            >{{ $t("extensions.connections.add") }}</UiButton
          >
        </div>
      </form>
      <p v-if="!connections.length" class="py-3 text-caption text-fg-secondary">
        {{ $t("extensions.connections.empty") }}
      </p>
      <div
        v-for="connection in connections"
        :key="connection.id"
        class="flex flex-wrap items-center gap-2 border-b border-stroke-divider py-3"
        :data-testid="`mcp-connection-${connection.id}`"
      >
        <div class="min-w-0 flex-1">
          <span class="block truncate text-body font-medium">{{ connection.name }}</span
          ><ExtensionAvailability
            kind="connections"
            :extension-id="connection.id"
            :enabled="connection.enabled"
            :plugin-id="connector.pluginId"
          />
        </div>
        <McpConnectionEnablement
          v-if="!connector.pluginId"
          :connection="connection"
          :configured="Boolean(connector.endpoint)"
          :disabled="busy || changes.isPending.value"
        />
        <UiButton
          size="sm"
          variant="ghost"
          :aria-expanded="selectedConnectionId === connection.id"
          @click="toggleSelected(connection.id)"
          >{{ $t("extensions.connections.accountsAndTools") }}</UiButton
        >
        <UiButton
          size="sm"
          variant="ghost"
          v-if="!connector.pluginId"
          class="text-fg-danger"
          :disabled="busy || changes.isPending.value"
          :data-testid="`delete-connection-${connection.id}`"
          @click="deleteConnection(connection)"
          >{{ $t("common.delete") }}</UiButton
        >
      </div>
    </section>
    <p v-if="changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(changes.error.value) }}
    </p>
    <section v-if="selected" class="space-y-3 border-t border-stroke-divider pt-4">
      <h4 class="text-body font-medium">
        {{ selected.name }} · {{ $t("extensions.connections.accountsAndTools") }}
      </h4>
      <McpConnectionSettings :key="selected.id" :connection="selected" :connector="connector" />
    </section>
  </div>
</template>
