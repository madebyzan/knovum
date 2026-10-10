<script setup lang="ts">
import { computed } from "vue";

import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import ConversationConnectionSelection from "./ConversationConnectionSelection.vue";
const selection = defineModel<AgentExtensionSelection>({ required: true });
const props = defineProps<{ connections: McpConnection[]; disabled: boolean }>();
const selectedConnections = computed(() =>
  props.connections.filter((connection) => isConnectionSelected(connection.id)),
);
function isConnectionSelected(connectionId: string) {
  return selection.value.connections.some((connection) => connection.connectionId === connectionId);
}
function toggleConnection(connection: McpConnection) {
  const id = connection.id;
  const selected = isConnectionSelected(id);
  selection.value = {
    ...selection.value,
    connections: selected
      ? selection.value.connections.filter((candidate) => candidate.connectionId !== id)
      : [
          ...selection.value.connections,
          {
            connectionId: id,
            toolNames: connection.tools.filter((tool) => tool.policy !== "deny").map((tool) => tool.name),
            resourceUris: [],
            resourceTemplates: [],
            promptNames: [],
          },
        ],
  };
}
</script>
<template>
  <section v-if="connections.length" class="space-y-1">
    <h3 class="pb-1 text-caption font-medium text-fg-tertiary">
      {{ $t("extensions.overview.mcp") }}
    </h3>
    <label
      v-for="connection in connections"
      :key="connection.id"
      class="flex cursor-default items-center gap-2.5 rounded-md p-2 text-label text-fg-primary hover:bg-surface-control-hover"
    >
      <UiCheckbox
        :disabled="disabled"
        :model-value="isConnectionSelected(connection.id)"
        :label="connection.name"
        @update:model-value="toggleConnection(connection)"
      />
      <span class="min-w-0 flex-1 truncate">{{ connection.name }}</span>
    </label>
    <ConversationConnectionSelection
      v-for="connection in selectedConnections"
      :key="connection.id"
      v-model="selection"
      :connection="connection"
      :disabled="disabled"
    />
  </section>
</template>
