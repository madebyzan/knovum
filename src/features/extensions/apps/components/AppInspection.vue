<script setup lang="ts">
import { Plug } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { connectionQuery, pluginQuery } from "../../../../foundation/extensions/extension-query";
import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import UiButton from "../../../../shared/ui/UiButton.vue";
import ExtensionInspectionHeader from "../../components/ExtensionInspectionHeader.vue";
import McpConnectionSettings from "../../connections/components/McpConnectionSettings.vue";
import ConnectorAccounts from "../../connectors/components/ConnectorAccounts.vue";
import PluginConnectorConfiguration from "../../connectors/components/PluginConnectorConfiguration.vue";
import type { ExtensionDestination } from "../../navigation/extension-destination";
import { usePluginLabels } from "../../plugins/use-plugin-labels";

const props = defineProps<{ connector: ConnectorDefinition }>();
const emit = defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const plugins = useQuery(pluginQuery);
const connections = useQuery(connectionQuery);
const plugin = computed(() =>
  plugins.data.value?.find((installation) => installation.id === props.connector.pluginId),
);
const appConnections = computed(() =>
  (connections.data.value ?? []).filter((connection) => connection.connectorId === props.connector.id),
);
const shouldConfigureEndpoint = computed(
  () =>
    !props.connector.endpoint ||
    props.connector.endpoint.type === "stdio" ||
    props.connector.endpoint.allowLocalNetwork,
);
const { describePlugin } = usePluginLabels();
const description = computed(() =>
  plugin.value?.source.type === "bundled"
    ? describePlugin(plugin.value.source, plugin.value.revision.manifest.name, props.connector.description)
        .description
    : props.connector.description,
);
const error = computed(() => plugins.error.value || connections.error.value);
</script>

<template>
  <section class="space-y-6" :data-testid="`app-inspection-${connector.id}`">
    <ExtensionInspectionHeader
      :name="connector.name"
      :description="description"
      :icon="Plug"
      @back="emit('navigate', { tab: 'apps' })"
    >
      <template #status
        ><p v-if="plugin && !plugin.enabled" class="mt-2 text-caption text-fg-warning">
          {{ $t("extensions.overview.pluginPaused") }}
        </p></template
      >
      <UiButton
        v-if="connector.pluginId"
        size="sm"
        variant="ghost"
        @click="emit('navigate', { tab: 'plugins', id: connector.pluginId })"
        >{{ $t("extensions.overview.sourcePlugin") }}</UiButton
      >
    </ExtensionInspectionHeader>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ getErrorMessage(error) }}</p>
    <PluginConnectorConfiguration v-if="shouldConfigureEndpoint" :connector="connector" />
    <section v-if="connector.endpoint && connector.authorization.type !== 'none'" class="space-y-3">
      <h3 class="border-b border-stroke-divider pb-2 text-body font-medium">
        {{ $t("extensions.apps.accounts") }}
      </h3>
      <ConnectorAccounts :connector="connector" />
    </section>
    <section v-if="connector.endpoint && appConnections.length === 1" class="space-y-3">
      <h3 class="text-body font-medium">{{ $t("extensions.apps.access") }}</h3>
      <McpConnectionSettings :connection="appConnections[0]!" :connector="connector" />
    </section>
    <p
      v-else-if="!connections.isPending.value && !connections.error.value"
      role="alert"
      class="text-caption text-fg-warning"
    >
      {{ $t("extensions.apps.configurationRequired") }}
    </p>
  </section>
</template>
