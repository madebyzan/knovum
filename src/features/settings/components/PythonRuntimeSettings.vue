<script setup lang="ts">
import { useI18n } from "vue-i18n";

import PythonRuntimeSettingsField from "./PythonRuntimeSettingsField.vue";
import ResourceDownloadSettings from "./ResourceDownloadSettings.vue";
import SettingsSection from "./SettingsSection.vue";
const { t } = useI18n();
</script>

<template>
  <SettingsSection
    class="mt-5"
    :title="t('settings.ai.pythonRuntimes.title')"
    :description="t('settings.ai.pythonRuntimes.description')"
  >
    <ResourceDownloadSettings />
    <PythonRuntimeSettingsField profile="inference" :label="t('settings.ai.pythonRuntimes.inference')" />
    <PythonRuntimeSettingsField profile="agent" :label="t('settings.ai.pythonRuntimes.agent')" />
  </SettingsSection>
</template>
