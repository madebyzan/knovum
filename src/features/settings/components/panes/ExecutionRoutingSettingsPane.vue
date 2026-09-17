<script setup lang="ts">
import { ask } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useExecutionCatalogQuery } from "../../../../foundation/ai/execution-catalog-query";
import UiButton from "../../../../shared/ui/UiButton.vue";
import AiRoutingSettings from "../AiRoutingSettings.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";

const { t } = useI18n();
const store = useAiSettingsStore();
const { settings, ready: settingsReady, loadError: settingsError, saveError } = storeToRefs(store);
const catalog = useExecutionCatalogQuery();
const ready = computed(() => settingsReady.value);
const loadError = computed(() => settingsError.value);

watch(saveError, (error) => {
  if (error) toast.error(`${t("settings.ai.integrationSaveFailed")}: ${String(error)}`);
});

async function load() {
  await store.ensureLoaded();
  if (catalog.descriptorError.value) await catalog.refetchDescriptors();
}

async function refreshInstallations() {
  const result = await catalog.refreshInstallations();
  if (result.error) toast.error(String(result.error));
}

async function resetSection() {
  const confirmed = await ask(t("settings.general.resetSectionConfirmation"), {
    title: t("settings.general.resetSection"),
    kind: "warning",
    okLabel: t("settings.general.resetSection"),
    cancelLabel: t("common.cancel"),
  });
  if (confirmed) await store.reset();
}

onMounted(load);
</script>

<template>
  <SettingsLoadFailure v-if="loadError" :message="String(loadError)" @retry="load" @reset="resetSection" />
  <div v-else :class="!ready && 'pointer-events-none opacity-60'">
    <p class="mb-5 max-w-2xl text-body text-foreground-secondary">
      {{ t("settings.ai.executionRoutingDescription") }}
    </p>
    <div
      v-if="catalog.descriptorError.value"
      class="bg-warning-subtle mb-4 flex items-center justify-between gap-3 rounded-control border border-warning/30 px-3 py-2"
    >
      <p class="min-w-0 truncate text-caption text-warning">
        {{ String(catalog.descriptorError.value) }}
      </p>
      <UiButton size="sm" variant="secondary" @click="catalog.refetchDescriptors">
        {{ t("common.retry") }}
      </UiButton>
    </div>
    <div
      v-if="catalog.installationError.value"
      class="bg-warning-subtle mb-4 flex items-center justify-between gap-3 rounded-control border border-warning/30 px-3 py-2"
    >
      <p class="min-w-0 truncate text-caption text-warning">{{ String(catalog.installationError.value) }}</p>
      <UiButton size="sm" variant="secondary" @click="refreshInstallations">
        {{ t("settings.ai.checkDesktopAgents") }}
      </UiButton>
    </div>
    <AiRoutingSettings
      v-model="settings.defaults"
      :connections="settings.connections"
      :desktop-agents="settings.desktopAgents"
      :agent-descriptors="catalog.data.value?.agents ?? []"
      :agent-installations="catalog.data.value?.installations ?? []"
      @changed="store.flush()"
    />
  </div>
</template>
