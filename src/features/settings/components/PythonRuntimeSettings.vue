<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import PythonRuntimeSettingsField from "./PythonRuntimeSettingsField.vue";
import ResourceDownloadSettings from "./ResourceDownloadSettings.vue";
const { t } = useI18n();
</script>

<template>
  <UiSettingsSection
    class="mt-5"
    :title="t('settings.ai.pythonRuntimes.title')"
    :description="t('settings.ai.pythonRuntimes.description')"
  >
    <ResourceDownloadSettings />
    <PythonRuntimeSettingsField profile="resources" :label="t('settings.ai.pythonRuntimes.resources')" />
    <PythonRuntimeSettingsField
      profile="light-inference"
      :label="t('settings.ai.pythonRuntimes.lightInference')"
    />
    <PythonRuntimeSettingsField
      profile="torch-inference"
      :label="t('settings.ai.pythonRuntimes.torchInference')"
    />
    <PythonRuntimeSettingsField
      profile="voice-generation"
      :label="t('settings.ai.pythonRuntimes.voiceGeneration')"
    />
    <PythonRuntimeSettingsField profile="agent" :label="t('settings.ai.pythonRuntimes.agent')" />
  </UiSettingsSection>
</template>
