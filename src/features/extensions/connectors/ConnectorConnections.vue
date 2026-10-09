<script setup lang="ts">
import { ChevronDown } from "@lucide/vue";
import { computed, shallowRef } from "vue";

import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiButton from "../../../shared/ui/UiButton.vue";
import ConnectorAccounts from "./ConnectorAccounts.vue";
import ConnectorInstances from "./ConnectorInstances.vue";
import PluginConnectorConfiguration from "./PluginConnectorConfiguration.vue";
const props = defineProps<{ connector: ConnectorDefinition; connections: McpConnection[]; busy: boolean }>();
defineEmits<{ edit: []; remove: [] }>();
const expanded = shallowRef(false);
const connections = computed(() =>
  props.connections.filter((connection) => connection.connectorId === props.connector.id),
);
</script>

<template>
  <section
    class="min-h-(--knovum-setting-min-height) min-w-0 px-4 py-3"
    :data-testid="`connector-${connector.id}`"
  >
    <header class="flex items-start gap-3">
      <div class="min-w-0 flex-1">
        <div class="flex flex-wrap items-center gap-2">
          <h3 class="text-body-lg font-medium break-words text-fg-primary">{{ connector.name }}</h3>
          <UiBadge size="sm">{{
            $t(connector.pluginId ? "extensions.connectors.plugin" : "extensions.connectors.custom")
          }}</UiBadge>
        </div>
        <p v-if="connector.description" class="mt-0.5 text-caption leading-normal text-fg-secondary">
          {{ connector.description }}
        </p>
        <p class="mt-1.5 text-caption text-fg-tertiary">
          {{ $t("extensions.connections.count", { count: connections.length }) }} · MCP
        </p>
      </div>
      <UiButton
        size="sm"
        variant="ghost"
        :aria-expanded="expanded"
        :aria-controls="`connector-connections-${connector.id}`"
        :data-testid="`manage-connector-${connector.id}`"
        @click="expanded = !expanded"
        >{{ $t(expanded ? "common.collapse" : "extensions.connections.manage")
        }}<ChevronDown
          class="size-3.5 transition-transform motion-reduce:transition-none"
          :class="expanded ? 'rotate-180' : ''"
      /></UiButton>
    </header>
    <div
      v-if="expanded"
      :id="`connector-connections-${connector.id}`"
      class="mt-4 space-y-4 border-t border-stroke-divider pt-4"
    >
      <div v-if="!connector.pluginId" class="flex flex-wrap gap-2">
        <UiButton size="sm" :disabled="busy" data-testid="edit-connector" @click="$emit('edit')">{{
          $t("extensions.connections.configure")
        }}</UiButton
        ><UiButton
          size="sm"
          variant="ghost"
          class="text-fg-danger"
          :disabled="busy"
          data-testid="delete-connector"
          @click="$emit('remove')"
          >{{ $t("common.delete") }}</UiButton
        >
      </div>
      <PluginConnectorConfiguration v-if="connector.pluginId" :connector="connector" />
      <section v-if="connector.endpoint && connector.authorization.type !== 'none'" class="space-y-3">
        <h4 class="text-body font-medium">{{ $t("extensions.connections.accounts") }}</h4>
        <ConnectorAccounts :connector="connector" />
      </section>
      <ConnectorInstances :connector="connector" :connections="connections" :busy="busy" />
    </div>
  </section>
</template>
