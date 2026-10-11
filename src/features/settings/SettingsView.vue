<script setup lang="ts">
import { computed, nextTick, toRaw, watch, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter, type LocationQueryRaw } from "vue-router";

import UiScrollArea from "../../shared/ui/UiScrollArea.vue";
import AboutSettingsPane from "./about/AboutSettingsPane.vue";
import AiModelsSettingsPane from "./ai/AiModelsSettingsPane.vue";
import LocalModelsSettings from "./ai/local-models/components/LocalModelsSettings.vue";
import AppearanceSettingsPane from "./appearance/AppearanceSettingsPane.vue";
import ArchivedSpacesSettingsPane from "./archives/ArchivedSpacesSettingsPane.vue";
import BackupSettingsPane from "./backup/BackupSettingsPane.vue";
import SettingsSidebar from "./components/SettingsSidebar.vue";
import type { GenerationActionOption } from "./creation/action-option";
import CreationSettingsPane from "./creation/CreationSettingsPane.vue";
import DesktopAgentsSettingsPane from "./desktop-agents/DesktopAgentsSettingsPane.vue";
import { resolveSettingsPaneId, type SettingsPaneId } from "./navigation/settings-pane";
import { getSettingsDefinition } from "./navigation/settings-registry";
import { useSettingsLeaveCoordinator } from "./navigation/use-settings-leave-coordinator";
import NotificationsSettingsPane from "./notifications/NotificationsSettingsPane.vue";
import UsageSettingsPane from "./usage/UsageSettingsPane.vue";

const props = defineProps<{
  loadGenerationActions: () => Promise<GenerationActionOption[]>;
  extensionsComponent: Component;
  voiceLibraryComponent: Component;
  returnToWorkspace: () => Promise<void>;
}>();
const { t } = useI18n();
const router = useRouter();
const route = useRoute();
useSettingsLeaveCoordinator();
const activePane = computed<SettingsPaneId>(() => {
  const value = Array.isArray(route.query.pane) ? route.query.pane[0] : route.query.pane;
  return resolveSettingsPaneId(value) ?? "appearance";
});

watch(
  () => route.query.pane,
  (value) => {
    const rawPane = Array.isArray(value) ? value[0] : value;
    const pane = resolveSettingsPaneId(rawPane) ?? "appearance";
    if (rawPane !== pane) {
      void router.replace({ path: route.path, query: { ...route.query, pane } });
    }
  },
  { immediate: true },
);

watch(
  () => [route.query.pane, route.query.setting] as const,
  async ([, setting]) => {
    const location = Array.isArray(setting) ? setting[0] : setting;
    if (!location) return;
    await nextTick();
    const element = document.getElementById(location);
    element?.scrollIntoView({ block: "center", behavior: "smooth" });
    element?.focus({ preventScroll: true });
  },
  { flush: "post", immediate: true },
);

const componentByPaneId = {
  aiModels: AiModelsSettingsPane,
  localModels: LocalModelsSettings,
  voices: toRaw(props.voiceLibraryComponent),
  desktopAgents: DesktopAgentsSettingsPane,
  extensions: toRaw(props.extensionsComponent),
  usage: UsageSettingsPane,
  creation: CreationSettingsPane,
  data: BackupSettingsPane,
  archivedSpaces: ArchivedSpacesSettingsPane,
  about: AboutSettingsPane,
  appearance: AppearanceSettingsPane,
  notifications: NotificationsSettingsPane,
} satisfies Record<SettingsPaneId, Component>;

const paneComponent = computed(() => componentByPaneId[activePane.value]);
const paneTitle = computed(() => t(getSettingsDefinition(activePane.value).titleKey));
const activeSetting = computed(() => {
  const value = Array.isArray(route.query.setting) ? route.query.setting[0] : route.query.setting;
  return typeof value === "string" ? value : undefined;
});

function selectPane(pane: SettingsPaneId, setting?: string) {
  if (activePane.value === pane && route.query.pane === pane && route.query.setting === setting) return;
  const query: LocationQueryRaw = { ...route.query, pane };
  if (pane !== "extensions") {
    delete query.extensionTab;
    delete query.extensionId;
    delete query.connectorId;
    delete query.extensionAction;
  }
  if (setting) query.setting = setting;
  else delete query.setting;
  void router.replace({ path: route.path, query });
}
</script>

<template>
  <main data-testid="settings-view" class="flex h-full min-h-0 bg-surface-canvas select-none">
    <SettingsSidebar
      :active-pane="activePane"
      :active-setting="activeSetting"
      @back="props.returnToWorkspace"
      @select="selectPane"
    />
    <div class="flex min-h-0 min-w-0 flex-1 flex-col">
      <div data-tauri-drag-region class="h-(--knovum-titlebar-height) shrink-0" />
      <div class="relative mt-4 min-h-0 flex-1">
        <UiScrollArea class="size-full select-text">
          <div class="mx-auto max-w-208 px-10 pt-10 pb-20">
            <header
              class="mb-5 flex min-h-(--knovum-control-height-sm) flex-wrap items-center justify-between gap-x-4 gap-y-2 px-0.5"
            >
              <h1 class="m-0 text-title font-semibold tracking-title text-fg-primary">
                {{ paneTitle }}
              </h1>
              <div id="settings-header-actions" class="ml-auto" />
            </header>
            <Transition
              mode="out-in"
              enter-active-class="animate-in fade-in-0 slide-in-from-right-1 duration-150 ease-standard motion-reduce:animate-none"
              leave-active-class="animate-out fade-out-0 duration-100 ease-standard motion-reduce:animate-none"
            >
              <div :key="activePane" class="space-y-6">
                <CreationSettingsPane
                  v-if="activePane === 'creation'"
                  :load-generation-actions="props.loadGenerationActions"
                />
                <component :is="paneComponent" v-else />
              </div>
            </Transition>
          </div>
        </UiScrollArea>
        <div
          class="pointer-events-none absolute inset-x-0 top-0 h-8 bg-linear-to-b from-surface-canvas to-transparent"
        />
        <div
          class="pointer-events-none absolute inset-x-0 bottom-0 h-12 bg-linear-to-t from-surface-canvas to-transparent"
        />
      </div>
    </div>
  </main>
</template>
