<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { usePreferencesStore } from "../../../foundation/preferences/preferences-store";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";
import PreferenceField from "../components/PreferenceField.vue";

const { t } = useI18n();
const preferences = usePreferencesStore();
const { backgroundTaskNotifications } = storeToRefs(preferences);

async function update(enabled: boolean) {
  try {
    await preferences.setBackgroundTaskNotifications(enabled);
  } catch (error) {
    toast.error(`${t("settings.notifications.saveFailed")}: ${getErrorMessage(error)}`);
  }
}
</script>

<template>
  <UiSettingsSection :description="t('settings.notifications.description')">
    <PreferenceField
      id="settings-background-task-notifications"
      tabindex="-1"
      :label="t('settings.notifications.backgroundTasks.label')"
      :description="t('settings.notifications.backgroundTasks.description')"
    >
      <UiSwitch
        :model-value="backgroundTaskNotifications"
        :label="t('settings.notifications.backgroundTasks.label')"
        @update:model-value="update"
      />
    </PreferenceField>
  </UiSettingsSection>
</template>
