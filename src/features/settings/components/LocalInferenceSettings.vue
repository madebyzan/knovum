<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { formatBytes } from "../../../foundation/format/bytes";
import {
  startResourcePreparation,
  type ResourcePreparationHandle,
} from "../../../foundation/runtime/resource-preparation";
import { useLocalResourcesQuery } from "../../../foundation/runtime/resource-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import PreferenceField from "./PreferenceField.vue";
import SettingsSection from "./SettingsSection.vue";

const { t } = useI18n();
const resources = useLocalResourcesQuery();
const preparation = shallowRef<ResourcePreparationHandle>();
const error = shallowRef<string>();
const runtime = computed(() => resources.data.value?.python);
const description = computed(() => {
  if (runtime.value?.available) return t("settings.ai.localInference.ready");
  if (!runtime.value?.downloadable) return t("settings.ai.localInference.unavailable");
  return t("settings.ai.localInference.download", { size: formatBytes(runtime.value.downloadSize!) });
});

async function prepare() {
  if (preparation.value) return;
  error.value = undefined;
  const handle = startResourcePreparation(
    { kind: "pythonRuntime" },
    t("settings.ai.localInference.preparing"),
  );
  preparation.value = handle;
  try {
    await handle.completion;
  } catch (cause) {
    if (decodeAppError(cause)?.code !== "local_resource_preparation_cancelled")
      error.value = getErrorMessage(cause);
  } finally {
    preparation.value = undefined;
  }
}

async function cancel() {
  try {
    await preparation.value?.cancel();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
</script>

<template>
  <SettingsSection
    class="mt-5"
    :title="t('settings.ai.localInference.title')"
    :description="t('settings.ai.localInference.description')"
  >
    <PreferenceField :label="t('settings.ai.localInference.components')" :description="description">
      <UiButton v-if="preparation" size="sm" variant="ghost" @click="cancel">{{
        t("settings.ai.localInference.cancel")
      }}</UiButton>
      <UiButton
        v-else-if="!runtime?.available"
        size="sm"
        variant="secondary"
        :disabled="!runtime?.downloadable"
        @click="prepare"
        >{{ t("settings.ai.localInference.prepare") }}</UiButton
      >
    </PreferenceField>
    <p v-if="error" role="alert" class="p-4 text-caption text-fg-danger">{{ error }}</p>
  </SettingsSection>
</template>
