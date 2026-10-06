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
import type { RuntimeProfile } from "../../../shared/contracts/generated/RuntimeProfile";
import UiButton from "../../../shared/ui/UiButton.vue";
import PreferenceField from "./PreferenceField.vue";

const props = defineProps<{ profile: RuntimeProfile; label: string }>();
const { t } = useI18n();
const resources = useLocalResourcesQuery();
const preparation = shallowRef<ResourcePreparationHandle>();
const error = shallowRef<string>();
const errorMessage = computed(
  () => error.value ?? (resources.error.value ? getErrorMessage(resources.error.value) : undefined),
);
const runtime = computed(() => resources.data.value?.pythonRuntimes[props.profile]);
const description = computed(() => {
  if (runtime.value?.available) return t("settings.ai.pythonRuntimes.ready");
  if (!runtime.value?.downloadable) return t("settings.ai.pythonRuntimes.unavailable");
  return t("settings.ai.pythonRuntimes.download", { size: formatBytes(runtime.value.downloadSize!) });
});

async function prepare() {
  if (preparation.value) return;
  error.value = undefined;
  const handle = startResourcePreparation(
    { kind: "pythonRuntime", profile: props.profile },
    t("settings.ai.pythonRuntimes.preparing", { name: props.label }),
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
  <PreferenceField :label="label" :description="description">
    <UiButton v-if="preparation" size="sm" variant="ghost" @click="cancel">{{
      t("settings.ai.pythonRuntimes.cancel")
    }}</UiButton>
    <UiButton
      v-else-if="!runtime?.available"
      size="sm"
      variant="secondary"
      :disabled="!runtime?.downloadable"
      @click="prepare"
      >{{ t("settings.ai.pythonRuntimes.prepare") }}</UiButton
    >
  </PreferenceField>
  <p v-if="errorMessage" role="alert" class="p-4 text-caption text-fg-danger">{{ errorMessage }}</p>
</template>
