<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { formatErrorDiagnostic } from "../../../../foundation/errors/error-format";
import { useUpdateStore } from "../../../../foundation/updates/update-store";
import UiButton from "../../../../shared/ui/UiButton.vue";
import PreferenceField from "../PreferenceField.vue";
import SettingsSection from "../SettingsSection.vue";

const { t } = useI18n();
const updates = useUpdateStore();
const version = shallowRef("");
const failureMessage = computed(() => {
  switch (updates.snapshot?.failure) {
    case "network": return t("settings.updates.failure.network");
    case "signature": return t("settings.updates.failure.signature");
    case "metadata": return t("settings.updates.failure.metadata");
    case "installation": return t("settings.updates.failure.installation");
    case "internal": return t("settings.updates.failure.internal");
    default: return "";
  }
});

onMounted(() => {
  void loadVersion();
  void updates.connect().catch((error) => {
    void logError(`Failed to observe application updates: ${formatErrorDiagnostic(error)}`);
  });
});
async function loadVersion() {
  try {
    version.value = await getVersion();
  } catch (error) {
    version.value = "—";
    void logError(`Failed to read application version: ${formatErrorDiagnostic(error)}`);
  }
}
</script>

<template>
  <div class="space-y-6">
    <SettingsSection :title="t('settings.general.about.title')">
      <PreferenceField :label="t('common.productName')" :description="
        version ? t('settings.general.version', { version }) : t('settings.general.readingVersion')
      " />
      <PreferenceField :label="t('settings.general.about.author')" description="Zan" />
    </SettingsSection>
    <SettingsSection :title="t('settings.updates.title')" :description="t('settings.updates.description')">
      <PreferenceField :label="updates.snapshot?.update
        ? t('settings.updates.available.title', { version: updates.snapshot.update.version })
        : t('settings.updates.channel.label')"
        :description="updates.snapshot?.configured
          ? t('settings.updates.channel.configured') : t('settings.updates.channel.notConfigured')">
        <div class="flex items-center gap-2">
          <UiButton data-testid="update-check" :disabled="updates.busy || (!updates.disconnected && !updates.snapshot?.configured)"
            @click="updates.check">
            {{ updates.phase === 'checking' ? t('settings.updates.check.checking') : t('settings.updates.check.action') }}
          </UiButton>
          <UiButton v-if="updates.phase === 'available'" data-testid="update-download" variant="primary"
            :disabled="updates.busy || updates.disconnected" @click="updates.download">
            {{ t('settings.updates.download.action') }}
          </UiButton>
          <UiButton v-if="updates.phase === 'downloading'" data-testid="update-cancel" @click="updates.cancel">
            {{ t('settings.updates.download.cancel') }}
          </UiButton>
          <UiButton v-if="updates.phase === 'ready'" data-testid="update-install" variant="primary"
            :disabled="updates.busy || updates.disconnected" @click="updates.install">
            {{ t('settings.updates.install.action') }}
          </UiButton>
        </div>
      </PreferenceField>
      <p v-if="updates.disconnected" role="alert">{{ t('settings.updates.unavailable') }}</p>
      <p v-if="failureMessage" role="alert">{{ failureMessage }}</p>
      <div v-if="updates.phase === 'downloading' || updates.phase === 'verifying'" role="status" aria-live="polite">
        <p>{{ updates.phase === 'verifying' ? t('settings.updates.download.verifying') : t('settings.updates.download.downloading') }}</p>
        <progress class="w-full" :value="updates.snapshot?.total ? updates.snapshot.downloaded : undefined"
          :max="updates.snapshot?.total || 1" :aria-label="t('settings.updates.download.downloading')" />
      </div>
      <p v-if="updates.phase === 'ready'" role="status">{{ t('settings.updates.download.ready') }}</p>
      <p v-if="updates.snapshot?.update?.notes" class="whitespace-pre-wrap text-body text-fg-secondary">
        {{ updates.snapshot.update.notes }}
      </p>
    </SettingsSection>
  </div>
</template>
