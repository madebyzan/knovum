<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { formatErrorDiagnostic } from "../../../../foundation/errors/error-format";
import { UPDATE_FAILURE_KEYS, UPDATE_STATUS_KEYS } from "../../../../foundation/updates/update-presentation";
import { useUpdateStore } from "../../../../foundation/updates/update-store";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import PreferenceField from "../PreferenceField.vue";

const { t } = useI18n();
const updates = useUpdateStore();
const version = shallowRef("");
const release = computed(() => (updates.state && "release" in updates.state ? updates.state.release : null));
const statusText = computed(() => {
  const state = updates.state;
  if (!state) return t("updates.connection.loading");
  return t(state.status === "failed" ? UPDATE_FAILURE_KEYS[state.reason] : UPDATE_STATUS_KEYS[state.status]);
});
const canCheck = computed(
  () => updates.state && ["idle", "current", "failed"].includes(updates.state.status),
);

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
      <PreferenceField
        :label="release ? t('updates.available.title', { version: release.version }) : t('updates.title')"
        :description="statusText"
      >
        <UiButton
          v-if="canCheck"
          data-testid="update-check"
          :disabled="updates.requestPending"
          @click="perform(updates.check)"
          >{{ t("updates.check.action") }}</UiButton
        >
        <UiButton
          v-else-if="updates.state?.status === 'available'"
          data-testid="update-download"
          variant="primary"
          :disabled="updates.requestPending"
          @click="perform(updates.download)"
          >{{ t("updates.download.action") }}</UiButton
        >
        <UiButton
          v-else-if="updates.state?.status === 'downloading'"
          data-testid="update-cancel"
          @click="perform(updates.cancelDownload)"
          >{{ t("updates.download.cancel") }}</UiButton
        >
        <UiButton
          v-else-if="updates.state?.status === 'ready'"
          data-testid="update-install"
          variant="primary"
          :disabled="!updates.canInstall"
          @click="perform(updates.install)"
          >{{ t("updates.install.action") }}</UiButton
        >
        <UiButton v-else-if="!updates.state" @click="perform(updates.initialize)">{{
          t("updates.connection.retry")
        }}</UiButton>
      </PreferenceField>
      <p v-if="updates.state?.status === 'ready' && updates.blockedByTasks" class="text-sm text-fg-secondary">
        {{ t("updates.install.blocked") }}
      </p>
      <progress
        v-if="updates.state?.status === 'downloading'"
        data-testid="update-progress"
        class="mt-3 h-2 w-full"
        :aria-label="t('updates.status.downloading')"
        :value="updates.state.total ? updates.state.downloaded : undefined"
        :max="updates.state.total || 1"
      />
      <div v-if="release?.notes" class="mt-4 space-y-2">
        <h3 class="text-sm font-medium text-fg-primary">{{ t("updates.available.notes") }}</h3>
        <p class="max-h-64 overflow-auto text-sm whitespace-pre-wrap text-fg-secondary">
          {{ release.notes }}
        </p>
      </div>
    </UiSettingsSection>
  </div>
</template>
