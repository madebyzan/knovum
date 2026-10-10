<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useLocalModelsQuery } from "../../../foundation/runtime/use-resource-queries";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSearchInput from "../../../shared/ui/UiSearchInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import { localModelGroups } from "../local-model-groups";
import LocalModelConfiguration from "./LocalModelConfiguration.vue";

const { t } = useI18n();
const models = useLocalModelsQuery();
const groups = computed(() => localModelGroups(models.data.value ?? []));
const capabilities = computed(() => groups.value.flatMap((group) => group.capabilities));
const capabilityOptions = computed(() =>
  groups.value.flatMap((group) =>
    group.capabilities.map((capability) => ({
      value: capability.id,
      label: t(`settings.ai.localModels.categoryLabels.${capability.id}`),
      description: t(`settings.ai.localModels.groups.${group.id}`),
    })),
  ),
);
const activeCapability = shallowRef("");
const searchQuery = shallowRef("");
const status = shallowRef("all");
const statusOptions = computed(() =>
  ["all", "installed", "notInstalled"].map((value) => ({
    value,
    label: t(`settings.ai.localModels.filters.${value}`),
  })),
);
watch(
  capabilities,
  (items) => {
    if (!items.some((item) => item.id === activeCapability.value))
      activeCapability.value = items[0]?.id ?? "";
  },
  { immediate: true },
);
watch(activeCapability, () => {
  searchQuery.value = "";
});
function matchesModelFilters(model: LocalModelAvailability) {
  const text = searchQuery.value.trim().toLocaleLowerCase();
  return (
    (status.value === "all" || model.installed === (status.value === "installed")) &&
    `${model.name} ${t(`settings.ai.localModels.modelDescriptions.${model.id}`)}`
      .toLocaleLowerCase()
      .includes(text)
  );
}
const visibleModels = computed(
  () => capabilities.value.find((item) => item.id === activeCapability.value)?.models.filter(matchesModelFilters) ?? [],
);
const visibleModelIds = computed(() => new Set(visibleModels.value.map((model) => model.id)));
</script>

<template>
  <section data-testid="local-models-settings-pane" class="space-y-6">
    <p class="text-caption leading-normal text-fg-secondary">
      {{ t("settings.ai.localModels.description") }}
    </p>
    <UiSettingsSection v-if="models.error.value || models.isPending.value">
      <div v-if="models.error.value" class="flex items-center justify-between gap-4 p-4" role="alert">
        <p class="text-caption text-fg-danger">{{ getErrorMessage(models.error.value) }}</p>
        <UiButton size="sm" @click="models.refetch()">{{ t("common.retry") }}</UiButton>
      </div>
      <p v-else class="p-4 text-caption text-fg-secondary" role="status">{{ t("common.loading") }}</p>
    </UiSettingsSection>
    <template v-else>
      <div class="flex flex-wrap items-center gap-2">
        <UiSelect
          v-model="activeCapability"
          data-testid="model-capability-select"
          class="w-44"
          :options="capabilityOptions"
          :label="t('settings.ai.localModels.categories')"
        />
        <UiSearchInput
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
      <UiSettingsSection
        v-for="capability in capabilities"
        :key="capability.id"
        v-show="activeCapability === capability.id"
        :data-model-capability="capability.id"
        :title="t(`settings.ai.localModels.categoryLabels.${capability.id}`)"
        :description="t(`settings.ai.localModels.capabilities.${capability.id}`)"
      >
        <LocalModelConfiguration
          v-for="model in capability.models"
          :key="model.id"
          v-show="visibleModelIds.has(model.id)"
          :model="model"
          :class="model.id !== visibleModels[0]?.id && 'border-t border-stroke-divider'"
        />
        <p
          v-if="!visibleModels.length"
          data-testid="model-no-results"
          class="px-4 py-6 text-caption text-fg-secondary"
        >
          {{ t("settings.ai.localModels.noResults") }}
        </p>
      </UiSettingsSection>
    </template>
  </section>
</template>
