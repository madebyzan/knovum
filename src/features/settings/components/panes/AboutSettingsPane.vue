<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { error as logError } from "@tauri-apps/plugin-log";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { formatErrorDiagnostic } from "../../../../foundation/errors/error-format";
import type { UpdateMetadata } from "../../../../shared/contracts/generated/UpdateMetadata";
import UiButton from "../../../../shared/ui/UiButton.vue";
import {
  checkForUpdate,
  getUpdateConfiguration,
  installUpdate as installApplicationUpdate,
} from "../../updater";
import PreferenceField from "../PreferenceField.vue";
import SettingsSection from "../SettingsSection.vue";

const { t } = useI18n();
const version = shallowRef("");
const updaterConfigured = shallowRef(false);
const checkingUpdate = shallowRef(false);
const installingUpdate = shallowRef(false);
const availableUpdate = shallowRef<UpdateMetadata | null>(null);

onMounted(() => {
  void loadVersion();
  void loadUpdateConfiguration();
});

async function loadVersion() {
  try {
    version.value = await getVersion();
  } catch (error) {
    version.value = "—";
    void logError(`Failed to read application version: ${formatErrorDiagnostic(error)}`);
  }
}

async function loadUpdateConfiguration() {
  try {
    updaterConfigured.value = (await getUpdateConfiguration()).configured;
  } catch (error) {
    void logError(`Failed to read updater status: ${formatErrorDiagnostic(error)}`);
  }
}

async function checkForUpdates() {
  if (checkingUpdate.value) return;
  checkingUpdate.value = true;
  try {
    availableUpdate.value = await checkForUpdate();
    if (!availableUpdate.value) toast.success(t("settings.updates.current.upToDate"));
  } catch (error) {
    toast.error(t("settings.updates.check.failed"));
    void logError(`Failed to check for application update: ${formatErrorDiagnostic(error)}`);
  } finally {
    checkingUpdate.value = false;
  }
}

async function installUpdate() {
  if (installingUpdate.value) return;
  installingUpdate.value = true;
  try {
    await installApplicationUpdate();
  } catch (error) {
    installingUpdate.value = false;
    toast.error(t("settings.updates.install.failed"));
    void logError(`Failed to install application update: ${formatErrorDiagnostic(error)}`);
  }
}
</script>

<template>
  <div class="space-y-6">
    <SettingsSection :title="t('settings.general.about.title')">
      <PreferenceField
        :label="t('common.productName')"
        :description="
          version ? t('settings.general.version', { version }) : t('settings.general.readingVersion')
        "
      />
      <PreferenceField :label="t('settings.general.about.author')" description="Zan" />
    </SettingsSection>

    <SettingsSection :title="t('settings.updates.title')" :description="t('settings.updates.description')">
      <PreferenceField
        :label="
          availableUpdate
            ? t('settings.updates.available.title', { version: availableUpdate.version })
            : t('settings.updates.channel.label')
        "
        :description="
          updaterConfigured
            ? availableUpdate
              ? t('settings.updates.available.description', { current: availableUpdate.currentVersion })
              : t('settings.updates.channel.connected')
            : t('settings.updates.channel.notConfigured')
        "
      >
        <UiButton
          v-if="availableUpdate"
          variant="primary"
          :disabled="installingUpdate"
          @click="installUpdate"
        >
          {{
            installingUpdate ? t("settings.updates.install.installing") : t("settings.updates.install.action")
          }}
        </UiButton>
        <UiButton v-else :disabled="!updaterConfigured || checkingUpdate" @click="checkForUpdates">
          {{ checkingUpdate ? t("settings.updates.check.checking") : t("settings.updates.check.action") }}
        </UiButton>
      </PreferenceField>
    </SettingsSection>
  </div>
</template>
