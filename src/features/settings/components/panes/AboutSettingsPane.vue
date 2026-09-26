<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { error as logError } from "@tauri-apps/plugin-log";
import { onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import {
  checkForUpdate,
  getUpdaterStatus,
  installUpdate as installApplicationUpdate,
  type UpdateMetadata,
} from "../../updater";
import SettingsRow from "../SettingsRow.vue";
import SettingsSection from "../SettingsSection.vue";

const { t } = useI18n();
const version = shallowRef("");
const updaterConfigured = shallowRef(false);
const checkingUpdate = shallowRef(false);
const installingUpdate = shallowRef(false);
const availableUpdate = shallowRef<UpdateMetadata | null>(null);

onMounted(() => {
  void loadVersion();
  void loadUpdaterStatus();
});

async function loadVersion() {
  try {
    version.value = await getVersion();
  } catch (error) {
    version.value = "—";
    void logError(`Failed to read application version: ${String(error)}`);
  }
}

async function loadUpdaterStatus() {
  try {
    updaterConfigured.value = (await getUpdaterStatus()).configured;
  } catch (error) {
    void logError(`Failed to read updater status: ${String(error)}`);
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
    void logError(`Failed to check for application update: ${String(error)}`);
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
    void logError(`Failed to install application update: ${String(error)}`);
  }
}
</script>

<template>
  <div class="space-y-6">
    <SettingsSection :title="t('settings.general.about.title')">
      <SettingsRow
        :label="t('common.productName')"
        :description="
          version ? t('settings.general.version', { version }) : t('settings.general.readingVersion')
        "
      />
    </SettingsSection>

    <SettingsSection :title="t('settings.updates.title')" :description="t('settings.updates.description')">
      <SettingsRow
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
      </SettingsRow>
    </SettingsSection>
  </div>
</template>
