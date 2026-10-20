<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../../foundation/errors/error-format";
import { useLocalModelsQuery } from "../../../../../foundation/runtime/use-local-models-query";
import type { LocalModelAvailability } from "../../../../../shared/contracts/generated/LocalModelAvailability";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiSearchInput from "../../../../../shared/ui/UiSearchInput.vue";
import UiSelect from "../../../../../shared/ui/UiSelect.vue";
import UiSettingsSection from "../../../../../shared/ui/UiSettingsSection.vue";
import { buildLocalModelGroups } from "../groups";
import LocalModelCapability from "./LocalModelCapability.vue";

const { t } = useI18n();
const models = useLocalModelsQuery();
const groups = computed(() => buildLocalModelGroups(models.data.value ?? []));
const searchQuery = shallowRef("");
const status = shallowRef("all");
const searchRef = useTemplateRef<InstanceType<typeof UiSearchInput>>("search");
const searchText = computed(() => searchQuery.value.trim().toLocaleLowerCase());
function clearFilters() {
  searchQuery.value = "";
  status.value = "all";
  searchRef.value?.focus();
}
const statusOptions = computed(() =>
  ["all", "installed", "notInstalled"].map((value) => ({
    value,
    label: t(`settings.ai.localModels.filters.${value}`),
  })),
);
function matchesModelFilters(model: LocalModelAvailability) {
  return (
    (status.value === "all" || model.installed === (status.value === "installed")) &&
    `${model.name} ${t(`settings.ai.localModels.modelDescriptions.${model.id}`)} ${t(`settings.ai.localModels.categoryLabels.${model.capability}`)}`
      .toLocaleLowerCase()
      .includes(searchText.value)
  );
}
const visibleModels = computed(() => (models.data.value ?? []).filter(matchesModelFilters));
const visibleModelIds = computed(() => new Set(visibleModels.value.map((model) => model.id)));
const filtering = computed(() => Boolean(searchQuery.value.trim()) || status.value !== "all");
const visibleCapabilities = computed(() => new Set(visibleModels.value.map((model) => model.capability)));
</script>

<template>
  <section data-testid="local-models-settings-pane" class="space-y-6">
    <p class="text-caption leading-normal text-fg-secondary">
      {{ t("settings.ai.localModels.description") }}
    </p>
    <UiSettingsSection v-if="models.error.value">
      <div class="flex flex-wrap items-center justify-between gap-4 p-4" role="alert">
        <p class="text-caption text-fg-danger">{{ getErrorMessage(models.error.value) }}</p>
        <UiButton
          size="sm"
          :disabled="models.isFetching.value"
          :aria-busy="models.isFetching.value"
          @click="models.refetch()"
          >{{ t("common.retry") }}</UiButton
        >
      </div>
    </UiSettingsSection>
    <UiSettingsSection v-if="models.isPending.value">
      <p class="p-4 text-caption text-fg-secondary" role="status">{{ t("common.loading") }}</p>
    </UiSettingsSection>
    <template v-else-if="models.data.value">
      <div class="flex flex-wrap items-center gap-2">
        <UiSearchInput
          ref="search"
          v-model="searchQuery"
          class="min-w-40 flex-1"
          :label="t('settings.ai.localModels.search')"
          :placeholder="t('settings.ai.localModels.search')"
        />
        <UiSelect
          v-model="status"
          data-testid="model-status-filter"
          class="w-32"
          :options="statusOptions"
          :label="t('settings.ai.localModels.filterLabel')"
        />
      </div>
      <section
        v-for="group in groups"
        :key="group.id"
        v-show="group.capabilities.some((capability) => visibleCapabilities.has(capability.id))"
        :data-model-group="group.id"
        class="space-y-2"
      >
        <h2 class="px-0.5 text-title-sm font-semibold text-fg-primary">
          {{ t(`settings.ai.localModels.groups.${group.id}`) }}
        </h2>
        <div class="space-y-2">
          <LocalModelCapability
            v-for="capability in group.capabilities"
            :key="capability.id"
            v-show="visibleCapabilities.has(capability.id)"
            :capability="capability.id"
            :models="capability.models"
            :visible-ids="visibleModelIds"
            :filtering="filtering"
          />
        </div>
      </section>
      <UiSettingsSection v-if="!visibleModels.length">
        <div class="flex flex-wrap items-center justify-between gap-3 px-4 py-6">
          <p data-testid="model-no-results" class="text-caption text-fg-secondary">
            {{ t("settings.ai.localModels.noResults") }}
          </p>
          <UiButton
            v-if="filtering"
            data-testid="model-clear-filters"
            size="sm"
            variant="ghost"
            @click="clearFilters"
          >
            {{ t("settings.ai.localModels.clearFilters") }}
          </UiButton>
        </div>
      </UiSettingsSection>
    </template>
  </section>
</template>
