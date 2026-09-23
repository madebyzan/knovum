<script setup lang="ts">
import { ArrowLeft } from "@lucide/vue";
import { compact } from "es-toolkit";
import { search } from "fast-fuzzy";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiScrollArea from "../../../shared/ui/UiScrollArea.vue";
import UiSearchInput from "../../../shared/ui/UiSearchInput.vue";
import type { SettingsPaneId } from "../settings-pane";
import { settingsGroupTitleKeys, settingsRegistry } from "../settings-registry";

const props = defineProps<{ activePane: SettingsPaneId; activeSetting?: string | undefined }>();
const emit = defineEmits<{ back: []; select: [pane: SettingsPaneId, location?: string] }>();
const { t } = useI18n();
const query = shallowRef("");
const groups = computed(() =>
  Object.entries(settingsGroupTitleKeys).map(([group, titleKey]) => ({
    id: group,
    label: t(titleKey),
    settings: settingsRegistry
      .filter((definition) => definition.group === group)
      .map((definition) => ({
        id: definition.id,
        icon: definition.icon,
        locations: definition.locations,
        label: t(definition.titleKey),
        description: t(definition.descriptionKey),
        location: undefined as string | undefined,
        searchText: [definition.titleKey, ...definition.searchKeys].map((key) => t(key)).join(" "),
      })),
  })),
);

function fuzzySettings<T extends { searchText: string }>(value: string, candidates: T[]) {
  return search(value, candidates, {
    keySelector: (setting) => setting.searchText,
    threshold: 0.8,
  });
}

const visibleGroups = computed(() => {
  const value = query.value.trim();
  if (!value) return groups.value;
  const normalizedValue = value.toLocaleLowerCase();
  const candidateGroups = groups.value.map((group) =>
    Object.assign({}, group, {
      settings: group.settings.flatMap((setting) =>
        setting.locations?.length
          ? setting.locations.map((location) => ({
              id: setting.id,
              icon: setting.icon,
              locations: setting.locations,
              location: location.id,
              label: t(location.labelKey),
              description: t(location.descriptionKey),
              searchText: `${t(location.labelKey)} ${t(location.descriptionKey)} ${setting.label}`,
            }))
          : [setting],
      ),
    }),
  );
  const hasExactMatch = candidateGroups.some((group) =>
    group.settings.some((setting) => setting.searchText.toLocaleLowerCase().includes(normalizedValue)),
  );

  return compact(
    candidateGroups.map((group) => {
      const settings = hasExactMatch
        ? group.settings.filter((setting) => setting.searchText.toLocaleLowerCase().includes(normalizedValue))
        : fuzzySettings(value, group.settings);
      return settings.length > 0 ? Object.assign({}, group, { settings }) : null;
    }),
  );
});
const searchResults = computed(() => visibleGroups.value.flatMap((group) => group.settings));
</script>

<template>
  <aside
    class="flex h-full w-(--knovum-settings-sidebar-width) shrink-0 flex-col border-r border-window-hairline bg-chrome"
  >
    <div data-tauri-drag-region class="h-(--knovum-window-chrome-height) shrink-0" />
    <div class="px-3">
      <button
        data-testid="settings-back"
        type="button"
        class="flex h-(--knovum-control-height-md) w-full items-center gap-2 rounded-control px-2 text-left text-label text-foreground-secondary transition-colors duration-100 hover:bg-(--knovum-sidebar-item-background-hover) hover:text-foreground"
        @click="emit('back')"
      >
        <ArrowLeft class="size-(--knovum-icon-size-sm)" :stroke-width="1.65" /><span>{{
          t("settings.general.backToApp")
        }}</span>
      </button>
      <UiSearchInput v-model="query" :placeholder="t('settings.general.search')" class="mt-2.5" />
    </div>
    <UiScrollArea class="mt-4 min-h-0 flex-1">
      <nav :aria-label="t('settings.general.title')" class="px-3 pb-6">
        <template v-if="query.trim()">
          <section v-for="group in visibleGroups" :key="group.id" class="mb-4">
            <p class="mb-1.5 px-2 text-caption font-medium text-foreground-tertiary">{{ group.label }}</p>
            <button
              v-for="setting in group.settings"
              :key="`${setting.id}:${setting.location ?? ''}`"
              type="button"
              class="mb-1 flex min-h-14 w-full items-start gap-2.5 rounded-control px-2 py-2 text-left transition-colors duration-100 hover:bg-(--knovum-sidebar-item-background-hover) data-[active=true]:bg-(--knovum-sidebar-item-background-selected)"
              :data-active="
                props.activePane === setting.id &&
                (setting.location ? props.activeSetting === setting.location : !props.activeSetting)
              "
              @click="emit('select', setting.id, setting.location)"
            >
              <component
                :is="setting.icon"
                class="mt-0.5 size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary"
                :stroke-width="1.6"
              />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-label font-medium text-foreground">{{ setting.label }}</span>
                <span class="mt-0.5 line-clamp-2 block text-caption leading-snug text-foreground-tertiary">
                  {{ setting.description }}
                </span>
              </span>
            </button>
          </section>
        </template>
        <template v-else>
          <section v-for="group in visibleGroups" :key="group.id" class="mb-4">
            <p class="mb-1.5 px-2 text-caption font-medium text-foreground-tertiary">{{ group.label }}</p>
            <button
              v-for="setting in group.settings"
              :key="setting.id"
              :data-testid="`settings-pane-${setting.id}`"
              type="button"
              class="mb-0.5 flex h-(--knovum-control-height-md) w-full items-center gap-2.5 rounded-control px-2 text-left text-body-lg text-foreground-secondary transition-colors duration-100 hover:bg-(--knovum-sidebar-item-background-hover) hover:text-foreground data-[active=true]:bg-(--knovum-sidebar-item-background-selected) data-[active=true]:font-medium data-[active=true]:text-foreground"
              :data-active="props.activePane === setting.id"
              @click="emit('select', setting.id)"
            >
              <component
                :is="setting.icon"
                class="size-(--knovum-icon-size-md) shrink-0"
                :stroke-width="1.6"
              /><span>{{ setting.label }}</span>
            </button>
          </section>
        </template>
        <p v-if="searchResults.length === 0" class="px-2 py-3 text-caption text-foreground-tertiary">
          {{ t("settings.general.noResults") }}
        </p>
      </nav>
    </UiScrollArea>
  </aside>
</template>
