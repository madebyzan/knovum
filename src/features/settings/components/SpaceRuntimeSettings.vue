<script setup lang="ts">
import { useI18n } from "vue-i18n";

import { useRuntimeSettings } from "../runtime/use-runtime-settings";
import RuntimeModelRow from "./RuntimeModelRow.vue";
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
    :title="t('settings.runtime.title')"
    :description="t('settings.runtime.description')"
  >
    <RuntimeModelRow
      :title="t('settings.runtime.textEmbeddingModel')"
      :description="t('settings.runtime.textEmbeddingModelDescription')"
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
    <RuntimeModelRow
      :title="t('settings.runtime.whisperStandardModel')"
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
