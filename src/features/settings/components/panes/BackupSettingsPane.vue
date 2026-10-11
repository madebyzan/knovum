<script setup lang="ts">
import { error as logError } from "@tauri-apps/plugin-log";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { formatErrorDiagnostic, getErrorMessage } from "../../../../foundation/errors/error-format";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import { useBackupOperations } from "../../backup/use-backup-operations";
import PreferenceField from "../PreferenceField.vue";

const { t } = useI18n();
const operations = useBackupOperations();

async function perform(operation: "backup" | "restore") {
  try {
    await operations[operation]();
  } catch (error) {
    if (isApplicationLeaveCancelled(error)) return;
    toast.error(t(`settings.data.${operation}.failed`), { description: getErrorMessage(error) });
    void logError(`Data ${operation} admission failed: ${formatErrorDiagnostic(error)}`);
  }
}
</script>

<template>
  <div class="space-y-6" data-testid="backup-settings-pane">
    <p class="max-w-2xl text-body text-fg-secondary">{{ t("settings.data.description") }}</p>
    <UiSettingsSection
      id="settings-data-backup"
      :title="t('settings.data.backup.title')"
      :description="t('settings.data.backup.description')"
    >
      <PreferenceField
        :label="t('settings.data.backup.contents.title')"
        :description="t('settings.data.backup.contents.description')"
      >
        <UiButton
          variant="primary"
          :disabled="Boolean(operations.busy.value) || operations.blockedByTasks.value"
          @click="perform('backup')"
          >{{ t("settings.data.backup.action") }}</UiButton
        >
      </PreferenceField>
    </UiSettingsSection>
    <UiSettingsSection
      id="settings-data-restore"
      :title="t('settings.data.restore.title')"
      :description="t('settings.data.restore.description')"
    >
      <PreferenceField
        :label="t('settings.data.restore.credentials.title')"
        :description="t('settings.data.restore.credentials.description')"
      >
        <UiButton
          variant="dangerSubtle"
          :disabled="Boolean(operations.busy.value) || operations.blockedByTasks.value"
          @click="perform('restore')"
          >{{ t("settings.data.restore.action") }}</UiButton
        >
      </PreferenceField>
    </UiSettingsSection>
  </div>
</template>
