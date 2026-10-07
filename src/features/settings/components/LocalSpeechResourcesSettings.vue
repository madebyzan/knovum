<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { formatBytes } from "../../../foundation/format/bytes";
import {
  removeDiarizationModel,
  setDiarizationAccessToken,
} from "../../../foundation/runtime/local-resources";
import {
  startResourcePreparation,
  type ResourcePreparationHandle,
} from "../../../foundation/runtime/resource-preparation";
import { invalidateLocalResources, useLocalResourcesQuery } from "../../../foundation/runtime/resource-query";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import PreferenceField from "./PreferenceField.vue";

const { t } = useI18n();
const resources = useLocalResourcesQuery();
const token = shallowRef("");
const busy = shallowRef(false);
const preparation = shallowRef<ResourcePreparationHandle>();
const error = shallowRef<string>();
const model = computed(() => resources.data.value?.models.diarization);
const description = computed(() =>
  t(
    model.value?.verified
      ? "settings.ai.localSpeech.ready"
      : model.value?.installed
        ? "settings.ai.localSpeech.invalid"
        : "settings.ai.localSpeech.notInstalled",
  ),
);

async function saveAccess(clear = false) {
  busy.value = true;
  error.value = undefined;
  try {
    await setDiarizationAccessToken(clear ? null : token.value.trim());
    token.value = "";
    await invalidateLocalResources();
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    busy.value = false;
  }
}

async function downloadModel() {
  error.value = undefined;
  const handle = startResourcePreparation(
    { kind: "speechDiarization" },
    t("settings.ai.localSpeech.downloading"),
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

async function remove() {
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

async function openOfficialPage(url: string) {
  try {
    await openUrl(url);
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
    <PreferenceField :label="t('settings.ai.localSpeech.diarization')" :description="description">
      <UiButton
        size="sm"
        variant="ghost"
        @click="openOfficialPage('https://huggingface.co/pyannote/speaker-diarization-community-1')"
        >{{ t("settings.ai.localSpeech.modelPage") }}</UiButton
      >
    </PreferenceField>
    <div class="space-y-3 border-t border-stroke-divider p-4">
      <p class="text-caption text-fg-secondary">{{ t("settings.ai.localSpeech.instructions") }}</p>
      <p v-if="model" class="text-caption text-fg-secondary">
        {{ t("settings.ai.localSpeech.modelSize", { size: formatBytes(model.totalBytes) }) }}
      </p>
      <label for="community-1-token" class="block text-caption text-fg-secondary">{{
        t("settings.ai.localSpeech.accessToken")
      }}</label>
      <UiInput
        id="community-1-token"
        v-model="token"
        type="password"
        autocomplete="off"
        :disabled="busy || Boolean(preparation)"
        :placeholder="
          t(
            model?.tokenConfigured
              ? 'settings.ai.keySavedPlaceholder'
              : 'settings.ai.localSpeech.tokenPlaceholder',
          )
        "
      />
      <div class="flex flex-wrap gap-2">
        <UiButton
          data-testid="community-1-save-access"
          size="sm"
          :disabled="busy || Boolean(preparation) || !token.trim()"
          @click="saveAccess()"
          >{{ t("settings.ai.localSpeech.saveToken") }}</UiButton
        >
        <UiButton
          v-if="model?.tokenConfigured"
          size="sm"
          variant="ghost"
          :disabled="busy || Boolean(preparation)"
          @click="saveAccess(true)"
          >{{ t("settings.ai.localSpeech.removeToken") }}</UiButton
        >
        <UiButton
          size="sm"
          variant="ghost"
          @click="openOfficialPage('https://huggingface.co/settings/tokens')"
          >{{ t("settings.ai.localSpeech.tokenPage") }}</UiButton
        >
      </div>
      <UiButton v-if="preparation" size="sm" variant="ghost" @click="cancel">{{
        t("common.cancel")
      }}</UiButton>
      <UiButton
        v-else-if="model && !model.installed"
        data-testid="community-1-download"
        size="sm"
        :disabled="busy || !model.tokenConfigured"
        @click="downloadModel"
        >{{ t("settings.ai.localSpeech.download") }}</UiButton
      >
      <UiButton
        v-if="model?.installed && !preparation"
        size="sm"
        variant="dangerSubtle"
        :disabled="busy"
        @click="remove"
        >{{ t("settings.ai.localSpeech.remove") }}</UiButton
      >
      <p v-if="busy" class="text-caption text-fg-tertiary" role="status">{{ t("common.loading") }}</p>
      <p v-if="error || resources.error.value" class="text-caption text-fg-danger" role="alert">
        {{ error ?? getErrorMessage(resources.error.value) }}
      </p>
    </div>
  </UiSettingsSection>
</template>
