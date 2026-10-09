<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { useLocalModelsQuery } from "../../../foundation/runtime/use-resource-queries";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import { localModelGroups } from "../local-model-groups";
import LocalModelConfiguration from "./LocalModelConfiguration.vue";

const { t } = useI18n();
const models = useLocalModelsQuery();
const groups = computed(() => localModelGroups(models.data.value ?? []));
</script>

<template>
  <section class="mt-6 space-y-5" aria-labelledby="local-models-title">
    <header class="px-0.5">
      <h2 id="local-models-title" class="text-title-sm font-semibold tracking-title text-fg-primary">
        {{ t("settings.ai.localModels.title") }}
      </h2>
      <p class="mt-1 max-w-2xl text-caption text-fg-secondary">
        {{ t("settings.ai.localModels.description") }}
      </p>
    </header>
    <UiSettingsSection v-if="models.error.value || models.isPending.value">
      <div v-if="models.error.value" class="flex items-center justify-between gap-4 p-4" role="alert">
        <p class="text-caption text-fg-danger">{{ getErrorMessage(models.error.value) }}</p>
        <UiButton size="sm" @click="models.refetch()">{{ t("common.retry") }}</UiButton>
      </div>
      <p v-else class="p-4 text-caption text-fg-secondary" role="status">{{ t("common.loading") }}</p>
    </UiSettingsSection>
    <section v-for="group in groups" v-else :key="group.id" :data-model-group="group.id" class="space-y-3">
      <h3 class="px-0.5 text-caption font-medium text-fg-tertiary">
        {{ t(`settings.ai.localModels.groups.${group.id}`) }}
      </h3>
      <UiSettingsSection
        v-for="capability in group.capabilities"
        :key="capability.id"
        :data-model-capability="capability.id"
        :title="t(`settings.ai.localModels.capabilities.${capability.id}`)"
      >
        <LocalModelConfiguration v-for="model in capability.models" :key="model.id" :model="model" />
      </UiSettingsSection>
    </section>
  </section>
</template>
