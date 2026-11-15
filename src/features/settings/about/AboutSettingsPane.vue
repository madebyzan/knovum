<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { error as logError } from "@tauri-apps/plugin-log";
import { storeToRefs } from "pinia";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useAnalyticsPreferencesStore } from "../../../foundation/analytics/analytics-preferences";
import { formatErrorDiagnostic } from "../../../foundation/errors/error-format";
import { useUpdateStore } from "../../../foundation/updates/update-store";
import { isApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";
import PreferenceField from "../components/PreferenceField.vue";
import SoftwareUpdate from "./SoftwareUpdate.vue";

const { t } = useI18n();
const updates = useUpdateStore();
const analytics = useAnalyticsPreferencesStore();
const { enabled: analyticsEnabled, ready: analyticsReady } = storeToRefs(analytics);
const analyticsStatus = shallowRef<"not_configured" | "unavailable" | undefined>();
const version = shallowRef("");
onMounted(() => {
  void loadVersion();
  void perform(updates.initialize);
});

async function updateAnalytics(enabled: boolean) {
  try {
    await analytics.setEnabled(enabled);
    const result = enabled ? await analytics.recordAppStarted() : "disabled";
    analyticsStatus.value = result === "not_configured" || result === "unavailable" ? result : undefined;
  } catch (error) {
    toast.error(t("settings.general.saveFailed"));
    void logError(`Analytics consent could not be saved: ${formatErrorDiagnostic(error)}`);
  }
}

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
    <UiSettingsSection>
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
    <UiSettingsSection
      :title="t('settings.general.analytics.title')"
      :description="t('settings.general.analytics.description')"
    >
      <PreferenceField
        :label="t('settings.general.analytics.enabled')"
        :description="t('settings.general.analytics.privacy')"
      >
        <UiSwitch
          :model-value="analyticsEnabled"
          :label="t('settings.general.analytics.enabled')"
          :disabled="!analyticsReady"
          @update:model-value="updateAnalytics"
        />
      </PreferenceField>
      <p v-if="analyticsStatus === 'not_configured'" class="px-4 text-caption text-fg-secondary">
        {{ t("settings.general.analytics.notConfigured") }}
      </p>
      <p v-else-if="analyticsStatus === 'unavailable'" class="px-4 text-caption text-fg-secondary">
        {{ t("settings.general.analytics.unavailable") }}
      </p>
    </UiSettingsSection>
    <UiSettingsSection :title="t('updates.title')" :description="t('updates.description')">
      <SoftwareUpdate @perform="perform" />
    </UiSettingsSection>
  </div>
</template>
