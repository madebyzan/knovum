<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { TabsRoot, TabsList, TabsTrigger, TabsContent } from "reka-ui";
import { computed } from "vue";

import {
  connectorQuery,
  connectionQuery,
  pluginQuery,
  skillQuery,
} from "../../foundation/extensions/extension-query";
import UiSearchInput from "../../shared/ui/UiSearchInput.vue";
import AppSettings from "./apps/components/AppSettings.vue";
import McpSettings from "./connections/components/McpSettings.vue";
import { isAppConnector } from "./connectors/app-connector";
import { EXTENSION_TABS } from "./navigation/extension-destination";
import { useExtensionNavigation } from "./navigation/use-extension-navigation";
import PluginSettings from "./plugins/components/PluginSettings.vue";
import SkillSettings from "./skills/components/SkillSettings.vue";

const { activeTab, extensionId, connectorId, action, isBrowsing, search, error, navigate, selectTab } =
  useExtensionNavigation();
const plugins = useQuery(pluginQuery);
const skills = useQuery(skillQuery);
const connections = useQuery(connectionQuery);
const connectors = useQuery(connectorQuery);
const pluginById = computed(() => new Map((plugins.data.value ?? []).map((plugin) => [plugin.id, plugin])));
const counts = computed(() => ({
  plugins: plugins.data.value?.length,
  apps: connectors.data.value?.filter((connector) =>
    isAppConnector(connector, pluginById.value.get(connector.pluginId!)),
  ).length,
  skills: skills.data.value?.filter((skill) => !skill.pluginId).length,
  mcp: connections.data.value?.filter((connection) =>
    connectors.data.value?.some(
      (connector) =>
        connector.id === connection.connectorId &&
        !isAppConnector(connector, pluginById.value.get(connector.pluginId!)),
    ),
  ).length,
}));
</script>

<template>
  <section data-testid="agent-extensions-settings" class="min-w-0 space-y-5">
    <p v-if="isBrowsing" class="max-w-xl text-body text-fg-secondary">
      {{ $t("extensions.overview.description") }}
    </p>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
    <TabsRoot :model-value="activeTab" @update:model-value="selectTab">
      <div v-if="isBrowsing" class="mb-6 flex flex-wrap items-center justify-between gap-3">
        <TabsList
          class="inline-flex max-w-full flex-wrap gap-1"
          :aria-label="$t('extensions.overview.title')"
        >
          <TabsTrigger
            v-for="tab in EXTENSION_TABS"
            :key="tab"
            :value="tab"
            :data-testid="`extensions-tab-${tab}`"
            class="inline-flex h-(--knovum-control-height-md) items-center justify-center gap-1.5 rounded-md px-2.5 text-label text-fg-secondary outline-none hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus data-[state=active]:bg-surface-control data-[state=active]:font-medium data-[state=active]:text-fg-primary"
          >
            {{ $t(`extensions.overview.${tab}`) }}
            <span v-if="counts[tab] !== undefined" class="text-caption text-fg-tertiary tabular-nums">{{
              counts[tab]
            }}</span>
          </TabsTrigger>
        </TabsList>
        <UiSearchInput
          v-model="search"
          class="w-56 max-w-full"
          :label="$t(`extensions.overview.search.${activeTab}`)"
          :placeholder="$t(`extensions.overview.search.${activeTab}`)"
          data-testid="extension-search"
        />
      </div>
      <TabsContent
        value="plugins"
        class="outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
        ><PluginSettings :search="search" :plugin-id="extensionId" :action="action" @navigate="navigate"
      /></TabsContent>
      <TabsContent
        value="apps"
        class="outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
        ><AppSettings :search="search" :connector-id="connectorId" @navigate="navigate"
      /></TabsContent>
      <TabsContent
        value="skills"
        class="outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
        ><SkillSettings :search="search" :skill-id="extensionId" @navigate="navigate"
      /></TabsContent>
      <TabsContent value="mcp" class="outline-none"
        ><McpSettings
          :search="search"
          :connection-id="extensionId"
          :connector-id="connectorId"
          :action="action === 'create' ? action : undefined"
          @navigate="navigate"
      /></TabsContent>
    </TabsRoot>
  </section>
</template>
