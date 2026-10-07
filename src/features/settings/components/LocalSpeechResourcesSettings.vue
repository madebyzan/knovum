<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { importDiarizationModel, removeDiarizationModel } from "../../../foundation/runtime/local-resources";
import { invalidateLocalResources, useLocalResourcesQuery } from "../../../foundation/runtime/resource-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import PreferenceField from "./PreferenceField.vue";

const { t } = useI18n();
const resources = useLocalResourcesQuery();
const accepted = shallowRef(false);
const busy = shallowRef(false);
const error = shallowRef<string>();
const installed = computed(() => resources.data.value?.models.diarization.verified === true);

async function chooseModel() {
  if (!accepted.value || busy.value) return;
  busy.value = true;
  error.value = undefined;
  try {
    const directory = await open({
      directory: true,
      multiple: false,
      title: t("settings.ai.localSpeech.import"),
    });
    if (!directory) return;
    await importDiarizationModel(directory, accepted.value);
    await invalidateLocalResources();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    busy.value = false;
  }
}
async function remove() {
  if (busy.value) return;
  busy.value = true;
  error.value = undefined;
  try {
    await removeDiarizationModel();
    await invalidateLocalResources();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    busy.value = false;
  }
}
async function openModelPage() {
  try {
    await openUrl("https://huggingface.co/pyannote/speaker-diarization-community-1");
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
</script>

<template>
  <UiSettingsSection
    class="mt-5"
    :title="t('settings.ai.localSpeech.title')"
    :description="t('settings.ai.localSpeech.description')"
  >
    <PreferenceField
      :label="t('settings.ai.localSpeech.diarization')"
      :description="t(installed ? 'settings.ai.localSpeech.ready' : 'settings.ai.localSpeech.notInstalled')"
    >
      <UiButton size="sm" variant="ghost" :disabled="busy" @click="openModelPage">{{
        t("settings.ai.localSpeech.modelPage")
      }}</UiButton>
    </PreferenceField>
    <div class="space-y-3 border-t border-stroke-divider p-4">
      <p class="text-caption text-fg-secondary">{{ t("settings.ai.localSpeech.instructions") }}</p>
      <label v-if="!installed" class="flex items-start gap-2 text-caption text-fg-secondary">
        <UiCheckbox v-model="accepted" :label="t('settings.ai.localSpeech.accepted')" :disabled="busy" />
        {{ t("settings.ai.localSpeech.accepted") }}
      </label>
      <UiButton v-if="!installed" size="sm" :disabled="busy || !accepted" @click="chooseModel">{{
        t("settings.ai.localSpeech.import")
      }}</UiButton>
      <UiButton v-else size="sm" variant="dangerSubtle" :disabled="busy" @click="remove">{{
        t("settings.ai.localSpeech.remove")
      }}</UiButton>
      <p v-if="busy" class="text-caption text-fg-tertiary" role="status">{{ t("common.loading") }}</p>
      <p v-if="error || resources.error.value" class="text-caption text-fg-danger" role="alert">
        {{ error ?? getErrorMessage(resources.error.value) }}
      </p>
    </div>
  </UiSettingsSection>
</template>
