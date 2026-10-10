<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { TabsRoot, TabsList, TabsTrigger, TabsContent } from "reka-ui";
import { computed, shallowRef } from "vue";
import {
  useRoute,
  useRouter,
  onBeforeRouteUpdate,
  isNavigationFailure,
  type LocationQueryRaw,
} from "vue-router";

import { getErrorMessage } from "../../foundation/errors/error-format";
import {
  connectorQuery,
  connectionQuery,
  pluginQuery,
  skillQuery,
} from "../../foundation/extensions/extension-query";
import { isApplicationLeaveCancelled } from "../../shared/contracts/application-leave";
import { useSettingsLeaveContext } from "../../shared/contracts/settings-leave";
import UiSearchInput from "../../shared/ui/UiSearchInput.vue";
import McpSettings from "./connections/McpSettings.vue";
import { EXTENSION_TABS, isExtensionTab, type ExtensionDestination } from "./extension-destination";
import PluginSettings from "./plugins/PluginSettings.vue";
import SkillSettings from "./skills/SkillSettings.vue";

const route = useRoute();
const router = useRouter();
const activeTab = computed(() =>
  isExtensionTab(route.query.extensionTab) ? route.query.extensionTab : "plugins",
);
const selectedId = computed(() =>
  typeof route.query.extensionId === "string" ? route.query.extensionId : undefined,
);
const connectorId = computed(() =>
  typeof route.query.connectorId === "string" ? route.query.connectorId : undefined,
);
const action = computed(() => (route.query.extensionAction === "create" ? ("create" as const) : undefined));
const inspecting = computed(() => Boolean(selectedId.value || connectorId.value || action.value));
const search = shallowRef("");
const error = shallowRef("");
const plugins = useQuery(pluginQuery);
const skills = useQuery(skillQuery);
const connections = useQuery(connectionQuery);
const connectors = useQuery(connectorQuery);
const counts = computed(() => ({
  plugins: plugins.data.value?.length,
  skills: skills.data.value?.filter((skill) => !skill.pluginId).length,
  mcp: connections.data.value?.filter((connection) =>
    connectors.data.value?.some(
      (connector) => connector.id === connection.connectorId && !connector.pluginId,
    ),
  ).length,
}));
const leaveCoordinator = useSettingsLeaveContext();

function isCurrentDestination(destination: ExtensionDestination) {
  return (
    destination.tab === activeTab.value &&
    destination.id === selectedId.value &&
    destination.connectorId === connectorId.value &&
    destination.action === action.value
  );
}
async function navigate(destination: ExtensionDestination) {
  if (isCurrentDestination(destination)) return;
  try {
    const query: LocationQueryRaw = { ...route.query, extensionTab: destination.tab };
    delete query.extensionId;
    delete query.setting;
    delete query.connectorId;
    delete query.extensionAction;
    if (destination.connectorId) query.connectorId = destination.connectorId;
    if (destination.action) query.extensionAction = destination.action;
    if (destination.id) query.extensionId = destination.id;
    const navigation = await router.push({ path: route.path, query });
    if (isNavigationFailure(navigation)) return;
    search.value = "";
    error.value = "";
  } catch (cause) {
    if (!isApplicationLeaveCancelled(cause)) error.value = getErrorMessage(cause);
  }
}
onBeforeRouteUpdate(async (to, from) => {
  if (to.query.pane !== from.query.pane) return true;
  if (
    to.query.extensionTab === from.query.extensionTab &&
    to.query.extensionId === from.query.extensionId &&
    to.query.connectorId === from.query.connectorId &&
    to.query.extensionAction === from.query.extensionAction
  )
    return true;
  try {
    await leaveCoordinator.requestLeave();
    return true;
  } catch (cause) {
    if (!isApplicationLeaveCancelled(cause)) error.value = getErrorMessage(cause);
    return false;
  }
});
function selectTab(value: string | number) {
  if (isExtensionTab(value)) void navigate({ tab: value });
}
</script>

<template>
  <section data-testid="agent-extensions-settings" class="min-w-0 space-y-5">
    <p v-if="!inspecting" class="max-w-xl text-body text-fg-secondary">
      {{ $t("extensions.overview.description") }}
    </p>
    <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
    <TabsRoot :model-value="activeTab" @update:model-value="selectTab">
      <div v-if="!inspecting" class="mb-6 flex flex-wrap items-center justify-between gap-3">
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
        ><PluginSettings :search="search" :selected-id="selectedId" @navigate="navigate"
      /></TabsContent>
      <TabsContent
        value="skills"
        class="outline-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
        ><SkillSettings :search="search" :selected-id="selectedId" @navigate="navigate"
      /></TabsContent>
      <TabsContent value="mcp" class="outline-none"
        ><McpSettings
          :search="search"
          :selected-id="selectedId"
          :connector-id="connectorId"
          :action="action"
          @navigate="navigate"
      /></TabsContent>
    </TabsRoot>
  </section>
</template>
