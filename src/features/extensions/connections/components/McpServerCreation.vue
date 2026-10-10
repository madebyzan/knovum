<script setup lang="ts">
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import ConnectorConfiguration from "../../connectors/components/ConnectorConfiguration.vue";
import { useConnectorEditing } from "../../connectors/use-connector-editing";
import type { ExtensionDestination } from "../../extension-destination";
const emit = defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const { changes, draft, endpointFields, edit, save } = useConnectorEditing();
edit();
async function create() {
  if (await save()) emit("navigate", { tab: "mcp" });
}
</script>
<template>
  <ConnectorConfiguration
    v-model="draft"
    v-model:endpoint-fields="endpointFields"
    :busy="changes.isPending.value"
    :title="$t('extensions.connections.addServer')"
    :error="changes.error.value ? getErrorMessage(changes.error.value) : undefined"
    @save="create"
    @close="$emit('navigate', { tab: 'mcp' })"
  />
</template>
