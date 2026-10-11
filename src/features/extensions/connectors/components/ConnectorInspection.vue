<script setup lang="ts">
import { Server } from "@lucide/vue";
import { computed } from "vue";

import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";
import UiButton from "../../../../shared/ui/UiButton.vue";
import ExtensionInspectionHeader from "../../components/ExtensionInspectionHeader.vue";
import type { ExtensionDestination } from "../../navigation/extension-destination";
import ConnectorAccounts from "./ConnectorAccounts.vue";
import ConnectorConnections from "./ConnectorConnections.vue";
import PluginConnectorConfiguration from "./PluginConnectorConfiguration.vue";
const props = defineProps<{ connector: ConnectorDefinition; connections: McpConnection[]; busy: boolean }>();
defineEmits<{ edit: []; delete: []; navigate: [destination: ExtensionDestination] }>();
const connectorConnections = computed(() =>
  props.connections.filter((connection) => connection.connectorId === props.connector.id),
);
</script>
<template>
  <section class="space-y-6" :data-testid="`connector-${connector.id}`">
    <ExtensionInspectionHeader
      :name="connector.name"
      :description="connector.description"
      :icon="Server"
      @back="
        $emit('navigate', connector.pluginId ? { tab: 'plugins', id: connector.pluginId } : { tab: 'mcp' })
      "
    >
      <template v-if="!connector.pluginId"
        ><UiButton size="sm" :disabled="busy" data-testid="edit-connector" @click="$emit('edit')">{{
          $t("extensions.connections.configure")
        }}</UiButton
        ><UiButton
          size="sm"
          variant="ghost"
          class="text-fg-danger"
          :disabled="busy"
          data-testid="delete-connector"
          @click="$emit('delete')"
          >{{ $t("common.delete") }}</UiButton
        ></template
      >
    </ExtensionInspectionHeader>
    <UiButton
      v-if="connector.pluginId"
      size="sm"
      variant="ghost"
      @click="$emit('navigate', { tab: 'plugins', id: connector.pluginId })"
      >{{ $t("extensions.overview.sourcePlugin") }}</UiButton
    >
    <PluginConnectorConfiguration v-if="connector.pluginId" :connector="connector" />
    <section v-if="connector.endpoint && connector.authorization.type !== 'none'" class="space-y-3">
      <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
        {{ $t("extensions.connections.accounts") }}
      </h3>
      <ConnectorAccounts :connector="connector" />
    </section>
    <ConnectorConnections :connector="connector" :connections="connectorConnections" :busy="busy" />
    <p class="text-caption text-fg-tertiary">{{ $t("extensions.accounts.billingHint") }}</p>
  </section>
</template>
