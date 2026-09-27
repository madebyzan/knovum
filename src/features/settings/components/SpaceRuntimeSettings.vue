<script setup lang="ts">
import { useI18n } from "vue-i18n";

import { useRuntimeSettings } from "../runtime/use-runtime-settings";
import RuntimeModelInstallation from "./RuntimeModelInstallation.vue";
import SettingsSection from "./SettingsSection.vue";

const { t } = useI18n();
const {
  busy,
  transcriptionInstalled,
  transcriptionInstalledCount,
  transcriptionSize,
  textEmbeddingInstalled,
  textEmbeddingInstalledCount,
  textEmbeddingTotalCount,
  textEmbeddingSize,
  transcriptionProgress,
  transcriptionTotalCount,
  textEmbeddingProgress,
  downloadTranscription,
  downloadTextEmbedding,
  removeTranscription,
  removeTextEmbedding,
  cancelDownload,
} = useRuntimeSettings();
</script>

<template>
  <SettingsSection
    id="settings-local-models"
    tabindex="-1"
    :title="t('settings.offlineModels.title')"
    :description="t('settings.offlineModels.description')"
  >
    <RuntimeModelInstallation
      :title="t('settings.offlineModels.models.textEmbedding.label')"
      :description="t('settings.offlineModels.models.textEmbedding.description')"
      :installed="textEmbeddingInstalled"
      :installed-count="textEmbeddingInstalledCount"
      :total-count="textEmbeddingTotalCount"
      :size="textEmbeddingSize"
      :progress="textEmbeddingProgress"
      :busy="Boolean(busy)"
      @download="downloadTextEmbedding"
      @cancel="cancelDownload"
      @remove="removeTextEmbedding"
    />
    <RuntimeModelInstallation
      :title="t('settings.offlineModels.models.whisperStandard')"
      :installed="transcriptionInstalled"
      :installed-count="transcriptionInstalledCount"
      :total-count="transcriptionTotalCount"
      :size="transcriptionSize"
      :progress="transcriptionProgress"
      :busy="Boolean(busy)"
      @download="downloadTranscription"
      @cancel="cancelDownload"
      @remove="removeTranscription"
    />
  </SettingsSection>
</template>
