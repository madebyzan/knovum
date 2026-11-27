<script setup lang="ts">
import { ArrowLeft, ChevronRight } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getUpdateAttentionVersion } from "../../../foundation/updates/update-presentation";
import { useUpdateStore } from "../../../foundation/updates/update-store";
import UiScrollArea from "../../../shared/ui/UiScrollArea.vue";
import UiSearchInput from "../../../shared/ui/UiSearchInput.vue";
import { buildSettingsNavGroups, searchSettingsNavGroups } from "./settings-navigation-groups";
import type { SettingsPaneId } from "./settings-pane";
import SettingsNavOption from "./SettingsNavOption.vue";

const props = defineProps<{ activePane: SettingsPaneId; activeSetting?: string | undefined }>();
const emit = defineEmits<{ back: []; select: [pane: SettingsPaneId, location?: string] }>();
const { t } = useI18n();
const updates = useUpdateStore();
const updateBadgeVersion = computed(() => getUpdateAttentionVersion(updates.state));
const query = shallowRef("");
const advancedExpanded = shallowRef(false);

const groups = computed(() => buildSettingsNavGroups(t));
const primaryGroups = computed(() =>
  groups.value
    .map((group) => ({ ...group, settings: group.settings.filter((setting) => !setting.advanced) }))
    .filter((group) => group.settings.length > 0),
);
const primaryGroupsBeforeAdvanced = computed(() =>
  primaryGroups.value.filter((group) => group.id !== "information"),
);
const informationGroups = computed(() => primaryGroups.value.filter((group) => group.id === "information"));
const advancedSettings = computed(() =>
  groups.value.flatMap((group) => group.settings.filter((setting) => setting.advanced)),
);
watch(
  () => props.activePane,
  (pane) => {
    if (advancedSettings.value.some((setting) => setting.id === pane)) advancedExpanded.value = true;
  },
  { immediate: true },
);

const visibleGroups = computed(() => searchSettingsNavGroups(groups.value, query.value, t));
const matchingSettings = computed(() => visibleGroups.value.flatMap((group) => group.settings));

function isSearchMatchActive(id: SettingsPaneId, location?: string) {
  return props.activePane === id && (location ? props.activeSetting === location : !props.activeSetting);
}
</script>

<template>
  <aside
    class="flex h-full w-62 shrink-0 flex-col border-r border-(--knovum-window-border-color) bg-surface-chrome"
  >
    <div data-tauri-drag-region class="h-(--knovum-titlebar-height) shrink-0" />
    <div class="px-3">
      <button
        data-testid="settings-back"
        type="button"
        class="flex h-(--knovum-control-height-md) w-full items-center gap-2 rounded-md px-2 text-left text-label text-fg-secondary transition-colors duration-100 hover:bg-(--knovum-sidebar-item-bg-hover) hover:text-fg-primary"
        @click="emit('back')"
      >
        <ArrowLeft class="size-3.5" :stroke-width="1.65" /><span>{{ t("settings.general.backToApp") }}</span>
      </button>
      <UiSearchInput
        v-model="query"
        :label="t('settings.general.search')"
        :placeholder="t('settings.general.search')"
        class="mt-2.5"
      />
    </div>
    <UiScrollArea class="mt-4 min-h-0 flex-1">
      <nav :aria-label="t('common.settings')" class="px-3 pb-6">
        <template v-if="query.trim()">
          <section v-for="group in visibleGroups" :key="group.id" class="mb-4">
            <p class="mb-1.5 px-2 text-caption font-medium text-fg-tertiary">{{ group.label }}</p>
            <button
              v-for="setting in group.settings"
              :key="`${setting.id}:${setting.location ?? ''}`"
              type="button"
              class="mb-1 flex min-h-14 w-full items-start gap-2.5 rounded-md px-2 py-2 text-left transition-colors duration-100 hover:bg-(--knovum-sidebar-item-bg-hover) data-[active=true]:bg-(--knovum-sidebar-item-bg-selected)"
              :data-active="isSearchMatchActive(setting.id, setting.location)"
              :aria-current="isSearchMatchActive(setting.id, setting.location) ? 'page' : undefined"
              @click="emit('select', setting.id, setting.location)"
            >
              <component
                :is="setting.icon"
                class="mt-0.5 size-3.5 shrink-0 text-fg-secondary"
                :stroke-width="1.6"
              />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-label font-medium text-fg-primary">{{ setting.label }}</span>
                <span class="mt-0.5 line-clamp-2 block text-caption leading-snug text-fg-tertiary">
                  {{ setting.description }}
                </span>
              </span>
            </button>
          </section>
        </template>
        <template v-else>
          <section v-for="group in primaryGroupsBeforeAdvanced" :key="group.id" class="mb-4">
            <p class="mb-1.5 px-2 text-caption font-medium text-fg-tertiary">{{ group.label }}</p>
            <SettingsNavOption
              v-for="setting in group.settings"
              :key="setting.id"
              :id="setting.id"
              :icon="setting.icon"
              :label="setting.label"
              :active="props.activePane === setting.id"
              @select="emit('select', $event)"
            />
          </section>
          <section v-if="advancedSettings.length" class="mb-4">
            <button
              type="button"
              class="mb-1 flex min-h-7 w-full items-center gap-1 rounded-md px-2 py-1.5 text-left text-caption font-medium text-fg-tertiary transition-colors hover:text-fg-secondary"
              :aria-expanded="advancedExpanded"
              aria-controls="settings-advanced-group"
              @click="advancedExpanded = !advancedExpanded"
            >
              <ChevronRight
                class="size-3 transition-transform duration-150 ease-standard"
                :class="advancedExpanded && 'rotate-90'"
              />
              <span>{{ t("settings.general.groups.advanced") }}</span>
            </button>
            <div v-if="advancedExpanded" id="settings-advanced-group">
              <SettingsNavOption
                v-for="setting in advancedSettings"
                :key="setting.id"
                :id="setting.id"
                :icon="setting.icon"
                :label="setting.label"
                :active="props.activePane === setting.id"
                @select="emit('select', $event)"
              />
            </div>
          </section>
          <section v-for="group in informationGroups" :key="group.id" class="mb-4">
            <p class="mb-1.5 px-2 text-caption font-medium text-fg-tertiary">{{ group.label }}</p>
            <SettingsNavOption
              v-for="setting in group.settings"
              :key="setting.id"
              :id="setting.id"
              :icon="setting.icon"
              :label="setting.label"
              :active="props.activePane === setting.id"
              :badge="setting.id === 'about' ? (updateBadgeVersion ?? undefined) : undefined"
              @select="emit('select', $event)"
            />
          </section>
        </template>
        <p
          v-if="matchingSettings.length === 0"
          role="status"
          aria-live="polite"
          class="px-2 py-3 text-caption text-fg-tertiary"
        >
          {{ t("settings.general.noResults") }}
        </p>
      </nav>
    </UiScrollArea>
  </aside>
</template>
