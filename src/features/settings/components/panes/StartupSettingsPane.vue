<script setup lang="ts">
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { error as logError } from "@tauri-apps/plugin-log";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiSwitch from "../../../../shared/ui/UiSwitch.vue";
import PreferenceField from "../PreferenceField.vue";
import SettingsSection from "../SettingsSection.vue";

const { t } = useI18n();
const autostart = shallowRef(false);

async function setAutostart(next: boolean) {
  try {
    if (next) await enable();
    else await disable();
    autostart.value = next;
  } catch (reason) {
    await logError(`Failed to change autostart: ${String(reason)}`);
    toast.error(t("settings.startup.launchFailed"));
  }
}

onMounted(() => {
  void isEnabled()
    .then((enabled) => (autostart.value = enabled))
    .catch((reason) => {
      void logError(`Failed to read autostart state: ${String(reason)}`);
      toast.error(t("settings.startup.launchFailed"));
    });
});
</script>

<template>
  <SettingsSection :title="t('settings.startup.title')">
    <PreferenceField
      :label="t('settings.startup.launchAtLogin.label')"
      :description="t('settings.startup.launchAtLogin.description')"
    >
      <UiSwitch
        :model-value="autostart"
        :label="t('settings.startup.launchAtLogin.label')"
        @update:model-value="setAutostart"
      />
    </PreferenceField>
  </SettingsSection>
</template>
