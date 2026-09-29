<script setup lang="ts">
import type { GenerationActionOption } from "./generation-action-option";

const props = defineProps<{ loadGenerationActions: () => Promise<GenerationActionOption[]> }>();
import { computed, nextTick, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter, type LocationQueryRaw } from "vue-router";

import { readLastNonSettingsRoute } from "../../foundation/navigation/workspace-session";
import UiScrollArea from "../../shared/ui/UiScrollArea.vue";
import AboutSettingsPane from "./components/panes/AboutSettingsPane.vue";
import AiModelsSettingsPane from "./components/panes/AiModelsSettingsPane.vue";
import AppearanceSettingsPane from "./components/panes/AppearanceSettingsPane.vue";
import ArchivedSpacesSettingsAction from "./components/panes/ArchivedSpacesSettingsAction.vue";
import ArchivedSpacesSettingsPane from "./components/panes/ArchivedSpacesSettingsPane.vue";
import CreationSettingsPane from "./components/panes/CreationSettingsPane.vue";
import DesktopAgentsSettingsAction from "./components/panes/DesktopAgentsSettingsAction.vue";
import DesktopAgentsSettingsPane from "./components/panes/DesktopAgentsSettingsPane.vue";
import UsageSettingsPane from "./components/panes/UsageSettingsPane.vue";
import SettingsSidebar from "./components/SettingsSidebar.vue";
import { resolveSettingsPaneId, type SettingsPaneId } from "./settings-pane";
import { settingsDefinition } from "./settings-registry";

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
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

const paneComponentMap = {
  aiModels: AiModelsSettingsPane,
  desktopAgents: DesktopAgentsSettingsPane,
  usage: UsageSettingsPane,
  creation: CreationSettingsPane,
  archivedSpaces: ArchivedSpacesSettingsPane,
  about: AboutSettingsPane,
  appearance: AppearanceSettingsPane,
};

const paneComponent = computed(() => paneComponentMap[activePane.value] ?? AppearanceSettingsPane);
const paneHeaderAction = computed(() =>
  activePane.value === "archivedSpaces"
    ? ArchivedSpacesSettingsAction
    : activePane.value === "desktopAgents"
      ? DesktopAgentsSettingsAction
      : undefined,
);
const paneTitle = computed(() => t(settingsDefinition(activePane.value).titleKey));
const activeSetting = computed(() => {
  const value = Array.isArray(route.query.setting) ? route.query.setting[0] : route.query.setting;
  return typeof value === "string" ? value : undefined;
});

function selectPane(pane: SettingsPaneId, setting?: string) {
  if (activePane.value === pane && route.query.pane === pane && route.query.setting === setting) return;
  const query: LocationQueryRaw = { ...route.query, pane };
  if (setting) query.setting = setting;
  else delete query.setting;
  void router.replace({ path: route.path, query });
}

function backToApp() {
  const target = readLastNonSettingsRoute();
  void router.push(target && router.resolve(target).matched.length ? target : "/spaces");
}
</script>

<template>
  <main data-testid="settings-view" class="flex h-full min-h-0 bg-surface-canvas select-none">
    <SettingsSidebar
      :active-pane="activePane"
      :active-setting="activeSetting"
      @back="backToApp"
      @select="selectPane"
    />
    <div class="flex min-h-0 min-w-0 flex-1 flex-col">
      <div data-tauri-drag-region class="h-(--knovum-titlebar-height) shrink-0" />
      <div class="relative mt-4 min-h-0 flex-1">
        <UiScrollArea class="size-full select-text">
          <div class="mx-auto max-w-208 px-10 pt-10 pb-20">
            <header
              class="mb-5 flex min-h-(--knovum-control-height-sm) items-center justify-between gap-4 px-0.5"
            >
              <h1 class="m-0 text-display font-semibold tracking-title text-fg-primary">
                {{ paneTitle }}
              </h1>
              <component :is="paneHeaderAction" v-if="paneHeaderAction" />
            </header>
            <Transition
              mode="out-in"
              enter-active-class="animate-in fade-in-0 slide-in-from-right-1 duration-150 ease-standard motion-reduce:animate-none"
              leave-active-class="animate-out fade-out-0 duration-100 ease-standard motion-reduce:animate-none"
            >
              <CreationSettingsPane
                v-if="activePane === 'creation'"
                :load-generation-actions="props.loadGenerationActions"
              />
              <component :is="paneComponent" v-else :key="activePane" />
            </Transition>
          </div>
        </UiScrollArea>
        <div class="pointer-events-none absolute inset-x-0 top-0 h-8 bg-linear-to-b from-surface-canvas to-transparent" />
        <div class="pointer-events-none absolute inset-x-0 bottom-0 h-12 bg-linear-to-t from-surface-canvas to-transparent" />
      </div>
    </div>
  </main>
</template>
