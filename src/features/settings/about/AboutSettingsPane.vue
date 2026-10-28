<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { error as logError } from "@tauri-apps/plugin-log";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { useUpdateStore } from "../../../foundation/updates/update-store";
import { isApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import PreferenceField from "../components/PreferenceField.vue";
import SoftwareUpdate from "./SoftwareUpdate.vue";

const { t } = useI18n();
const updates = useUpdateStore();
const version = shallowRef("");
onMounted(() => {
  void loadVersion();
  void perform(updates.initialize);
});

async function loadVersion() {
  try {
    version.value = await getVersion();
  } catch (error) {
    version.value = "—";
    void logError(`Failed to read application version: ${formatErrorDiagnostic(error)}`);
  }
}

async function perform(action: () => Promise<unknown>) {
  try {
    await action();
  } catch (error) {
    if (isApplicationLeaveCancelled(error)) return;
    toast.error(t("updates.action.failed"));
    void logError(`Application update action failed: ${formatErrorDiagnostic(error)}`);
  }
}
</script>

<template>
  <div class="space-y-6" data-testid="about-settings-pane">
    <UiSettingsSection :title="t('settings.general.about.title')">
      <PreferenceField
        :label="t('common.productName')"
        :description="
          version
            ? t('settings.general.about.version.value', { version })
            : t('settings.general.about.version.loading')
        "
      />
      <PreferenceField :label="t('settings.general.about.author')" description="Zan" />
    </UiSettingsSection>
    <UiSettingsSection :title="t('updates.title')" :description="t('updates.description')">
      <SoftwareUpdate @perform="perform" />
    </UiSettingsSection>
  </div>
</template>
