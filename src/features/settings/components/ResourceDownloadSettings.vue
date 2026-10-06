<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { setResourceDownloadRegion } from "../../../foundation/runtime/local-resources";
import { invalidateLocalResources, useLocalResourcesQuery } from "../../../foundation/runtime/resource-query";
import type { ResourceDownloadRegion } from "../../../shared/contracts/generated/ResourceDownloadRegion";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import PreferenceField from "./PreferenceField.vue";

const { t } = useI18n();
const resources = useLocalResourcesQuery();
const saving = shallowRef(false);
const error = shallowRef<string>();
const options = computed(() => [
  { value: "global" as const, label: t("settings.ai.resourceDownloads.global") },
  { value: "china" as const, label: t("settings.ai.resourceDownloads.china") },
]);

async function update(region: ResourceDownloadRegion) {
  saving.value = true;
  error.value = undefined;
  try {
    await setResourceDownloadRegion(region);
    await invalidateLocalResources();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <PreferenceField
    :label="t('settings.ai.resourceDownloads.label')"
    :description="t('settings.ai.resourceDownloads.description')"
  >
    <UiSelect
      v-if="resources.data.value"
      :model-value="resources.data.value.downloadRegion"
      :options="options"
      :disabled="saving"
      :label="t('settings.ai.resourceDownloads.label')"
      @update:model-value="update"
    />
  </PreferenceField>
  <p v-if="resources.data.value?.downloadRegion === 'china'" class="px-4 text-caption text-fg-secondary">
    {{ t("settings.ai.resourceDownloads.chinaNote") }}
  </p>
  <p v-if="error" role="alert" class="px-4 text-caption text-fg-danger">{{ error }}</p>
</template>
