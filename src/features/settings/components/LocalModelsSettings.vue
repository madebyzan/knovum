<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useLocalModelsQuery } from "../../../foundation/runtime/resource-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import { localModelGroups } from "../local-model-groups";
import LocalModelConfiguration from "./LocalModelConfiguration.vue";

const { t } = useI18n();
const models = useLocalModelsQuery();
const groups = computed(() => localModelGroups(models.data.value ?? []));
</script>

<template>
  <UiSettingsSection
    class="mt-5"
    :title="t('settings.ai.localModels.title')"
    :description="t('settings.ai.localModels.description')"
  >
    <div v-if="models.error.value" class="space-y-2 p-4" role="alert">
      <p class="text-caption text-fg-danger">{{ getErrorMessage(models.error.value) }}</p>
      <UiButton size="sm" variant="ghost" @click="models.refetch()">{{ t("common.retry") }}</UiButton>
    </div>
    <p v-else-if="models.isPending.value" class="p-4 text-caption text-fg-secondary" role="status">
      {{ t("common.loading") }}
    </p>
    <div v-else class="space-y-4 p-4">
      <section v-for="group in groups" :key="group.id" :data-model-group="group.id" class="space-y-3">
        <h2 class="text-label font-semibold text-fg-primary">
          {{ t(`settings.ai.localModels.groups.${group.id}`) }}
        </h2>
        <section
          v-for="capability in group.capabilities"
          :key="capability.id"
          class="overflow-hidden rounded-panel border border-stroke-divider"
          :data-model-capability="capability.id"
        >
          <h3 class="bg-surface-muted px-4 py-2 text-caption font-medium text-fg-secondary">
            {{ t(`settings.ai.localModels.capabilities.${capability.id}`) }}
          </h3>
          <LocalModelConfiguration v-for="model in capability.models" :key="model.id" :model="model" />
        </section>
      </section>
    </div>
  </UiSettingsSection>
</template>
