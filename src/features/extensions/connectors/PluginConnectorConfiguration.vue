<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, shallowRef, watch } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { configurePluginConnector } from "../../../foundation/extensions/connector-operations";
import { pluginQuery } from "../../../foundation/extensions/extension-query";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { PluginConnectorConfiguration } from "../../../shared/contracts/generated/PluginConnectorConfiguration";
import UiButton from "../../../shared/ui/UiButton.vue";
import ExtensionCleanupNotice from "../ExtensionCleanupNotice.vue";
import { useExtensionChanges } from "../use-extension-changes";

const props = defineProps<{ connector: ConnectorDefinition }>();
const plugins = useQuery(pluginQuery);
const changes = useExtensionChanges();
const localNetwork = shallowRef(false);
const declaration = computed(() =>
  plugins.data.value
    ?.find((plugin) => plugin.id === props.connector.pluginId)
    ?.revision.connectors.find((component) => component.componentName === props.connector.componentName),
);
const configuration = computed(
  () =>
    declaration.value?.configuration as
      | { type: string; command?: string; args?: string[]; url?: string }
      | undefined,
);
watch(
  () => props.connector.revision,
  () => {
    localNetwork.value =
      props.connector.endpoint?.type === "streamable_http" && props.connector.endpoint.allowLocalNetwork;
  },
  { immediate: true },
);
function save(selection: PluginConnectorConfiguration) {
  return changes.mutateAsync(() => configurePluginConnector(props.connector, selection));
}
async function chooseExecutable() {
  await changes
    .mutateAsync(async () => {
      const path = await open({ directory: false, multiple: false });
      if (typeof path === "string")
        return configurePluginConnector(props.connector, { type: "stdio", executable: path });
    })
    .catch(() => {});
}
</script>

<template>
  <section class="space-y-2 border-t border-stroke-divider pt-2" data-testid="plugin-connector-configuration">
    <p v-if="plugins.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(plugins.error.value || changes.error.value) }}
    </p>
    <template v-if="configuration?.type === 'stdio'">
      <p class="text-caption text-fg-secondary">
        {{ $t("extensions.declaredCommand") }}: {{ configuration.command }}
      </p>
      <p v-if="configuration.args?.length" class="text-caption break-all text-fg-secondary">
        {{ configuration.args.join(" ") }}
      </p>
      <p class="text-caption break-all" :class="connector.endpoint ? 'text-fg-secondary' : 'text-fg-warning'">
        {{
          connector.endpoint?.type === "stdio"
            ? connector.endpoint.executable
            : $t("extensions.localConfigurationRequired")
        }}
      </p>
      <p class="text-caption text-fg-warning">{{ $t("extensions.explicitProcessAuthorization") }}</p>
      <UiButton
        size="sm"
        :disabled="changes.isPending.value"
        data-testid="bind-plugin-executable"
        @click="chooseExecutable"
      >
        {{ $t("extensions.chooseInstalledExecutable") }}
      </UiButton>
    </template>
    <form
      v-else-if="configuration?.type === 'streamable-http'"
      class="space-y-2"
      @submit.prevent="save({ type: 'streamable_http', allowLocalNetwork: localNetwork }).catch(() => {})"
    >
      <p class="text-caption break-all text-fg-secondary">{{ configuration.url }}</p>
      <label class="flex items-center gap-2 text-caption"
        ><input v-model="localNetwork" type="checkbox" />{{ $t("extensions.allowLocalNetwork") }}</label
      >
      <UiButton type="submit" size="sm" :disabled="changes.isPending.value">{{
        $t("extensions.save")
      }}</UiButton>
    </form>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
  </section>
</template>
