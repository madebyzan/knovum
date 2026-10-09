<script setup lang="ts">
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import { usePluginConnectorConfiguration } from "./use-plugin-connector-configuration";
const props = defineProps<{ connector: ConnectorDefinition }>();
const { plugins, changes, configuration, localNetwork, hasConflict, save, chooseExecutable } =
  usePluginConnectorConfiguration(() => props.connector);
</script>

<template>
  <section class="space-y-2 border-t border-stroke-divider pt-2" data-testid="plugin-connector-configuration">
    <p v-if="plugins.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(plugins.error.value || changes.error.value) }}
    </p>
    <template v-if="configuration?.type === 'stdio'">
      <p class="text-caption text-fg-secondary">
        {{ $t("extensions.connectors.process.declaredCommand") }}: {{ configuration.command }}
      </p>
      <p v-if="configuration.args?.length" class="text-caption break-all text-fg-secondary">
        {{ configuration.args.join(" ") }}
      </p>
      <p class="text-caption break-all" :class="connector.endpoint ? 'text-fg-secondary' : 'text-fg-warning'">
        {{
          connector.endpoint?.type === "stdio"
            ? connector.endpoint.executable
            : $t("extensions.connectors.process.configurationRequired")
        }}
      </p>
      <p class="text-caption text-fg-warning">{{ $t("extensions.connectors.process.authorizationHint") }}</p>
      <UiButton
        size="sm"
        :disabled="changes.isPending.value || hasConflict"
        data-testid="bind-plugin-executable"
        @click="chooseExecutable"
      >
        {{ $t("extensions.connectors.process.chooseExecutable") }}
      </UiButton>
    </template>
    <form
      v-else-if="configuration?.type === 'streamable-http'"
      class="space-y-2"
      @submit.prevent="save({ type: 'streamable_http', allowLocalNetwork: localNetwork })"
    >
      <p class="text-caption break-all text-fg-secondary">{{ configuration.url }}</p>
      <label class="flex items-center gap-2 text-caption"
        ><UiCheckbox
          v-model="localNetwork"
          :disabled="changes.isPending.value || hasConflict"
          :label="$t('extensions.connections.allowLocalNetwork')"
        />{{ $t("extensions.connections.allowLocalNetwork") }}</label
      >
      <UiButton type="submit" size="sm" :disabled="changes.isPending.value || hasConflict">{{
        $t("common.save")
      }}</UiButton>
    </form>
    <p v-if="hasConflict" role="alert" class="text-caption text-fg-warning">
      {{ $t("extensions.connections.conflict") }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
  </section>
</template>
