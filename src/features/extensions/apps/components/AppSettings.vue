<script setup lang="ts">
import { Compass, Plug } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import {
  connectionQuery,
  connectorQuery,
  pluginQuery,
} from "../../../../foundation/extensions/extension-query";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSettingsHeaderActions from "../../../../shared/ui/UiSettingsHeaderActions.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import ExtensionAvailability from "../../components/ExtensionAvailability.vue";
import ExtensionEmptyState from "../../components/ExtensionEmptyState.vue";
import ExtensionIdentity from "../../components/ExtensionIdentity.vue";
import { isAppConnector } from "../../connectors/app-connector";
import type { ExtensionDestination } from "../../extension-destination";
import { usePluginLabels } from "../../plugins/use-plugin-labels";
import AppInspection from "./AppInspection.vue";

const props = defineProps<{ search: string; connectorId?: string | undefined }>();
const emit = defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const connectors = useQuery(connectorQuery);
const connections = useQuery(connectionQuery);
const plugins = useQuery(pluginQuery);
const apps = computed(() =>
  (connectors.data.value ?? []).filter((connector) =>
    isAppConnector(connector, pluginById.value.get(connector.pluginId!)),
  ),
);
const visibleApps = computed(() =>
  apps.value.filter((connector) =>
    `${connector.name} ${connector.description} ${descriptionByConnectorId.value.get(connector.id) ?? ""}`
      .toLowerCase()
      .includes(props.search.trim().toLowerCase()),
  ),
);
const selectedApp = computed(() => apps.value.find((connector) => connector.id === props.connectorId));
const connectionByConnectorId = computed(
  () => new Map((connections.data.value ?? []).map((connection) => [connection.connectorId, connection])),
);
const pluginById = computed(() => new Map((plugins.data.value ?? []).map((plugin) => [plugin.id, plugin])));
const { describePlugin } = usePluginLabels();
const descriptionByConnectorId = computed(
  () =>
    new Map(
      apps.value.map((connector) => {
        const owner = pluginById.value.get(connector.pluginId!);
        const description =
          owner?.source.type === "bundled"
            ? describePlugin(owner.source, owner.revision.manifest.name, connector.description).description
            : connector.description;
        return [connector.id, description];
      }),
    ),
);
const error = computed(() => connectors.error.value || connections.error.value || plugins.error.value);
const isLoading = computed(
  () => connectors.isPending.value || connections.isPending.value || plugins.isPending.value,
);
</script>

<template>
  <section class="space-y-4">
    <UiSettingsHeaderActions v-if="!connectorId">
      <UiButton
        size="sm"
        data-testid="browse-app-plugins"
        @click="emit('navigate', { tab: 'plugins', action: 'browse' })"
        ><Compass class="size-4" />{{ $t("extensions.apps.browse") }}</UiButton
      >
    </UiSettingsHeaderActions>
    <p v-if="error" role="alert" class="text-body text-fg-danger">{{ getErrorMessage(error) }}</p>
    <AppInspection
      v-if="selectedApp"
      :key="selectedApp.id"
      :connector="selectedApp"
      @navigate="emit('navigate', $event)"
    />
    <div
      v-else-if="isLoading"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="connectorId && !error"
      :title="$t('extensions.overview.notFound.title')"
      :description="$t('extensions.overview.notFound.description')"
      ><UiButton @click="emit('navigate', { tab: 'apps' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <template v-else-if="!connectorId">
      <p class="text-caption text-fg-secondary">{{ $t("extensions.apps.description") }}</p>
      <ExtensionEmptyState
        v-if="!error && !visibleApps.length"
        :title="$t(search ? 'common.search.empty' : 'extensions.apps.empty.title')"
        :description="$t(search ? 'common.search.hint' : 'extensions.apps.empty.description')"
      />
      <UiSettingsSection v-if="visibleApps.length">
        <div class="divide-y divide-stroke-divider px-4">
          <ExtensionIdentity
            v-for="connector in visibleApps"
            :key="connector.id"
            :name="connector.name"
            :description="descriptionByConnectorId.get(connector.id)"
            :icon="Plug"
            :data-testid="`app-${connector.id}`"
            :muted="!pluginById.get(connector.pluginId!)?.enabled"
            @inspect="emit('navigate', { tab: 'apps', connectorId: connector.id })"
          >
            <template #status
              ><ExtensionAvailability
                v-if="connectionByConnectorId.get(connector.id)"
                class="mt-1"
                kind="connections"
                :extension-id="connectionByConnectorId.get(connector.id)!.id"
                :enabled="Boolean(pluginById.get(connector.pluginId!)?.enabled)"
                :plugin-id="connector.pluginId"
            /></template>
            <UiButton
              size="sm"
              variant="ghost"
              @click="emit('navigate', { tab: 'apps', connectorId: connector.id })"
              >{{ $t("extensions.apps.manage") }}</UiButton
            >
          </ExtensionIdentity>
        </div>
      </UiSettingsSection>
    </template>
  </section>
</template>
