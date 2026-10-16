<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { useCreationLibraryStore } from "../../../foundation/ai/creation-library-store";
import { setDefaultExecutionTarget } from "../../../foundation/ai/execution-settings";
import { resetAiSettings } from "../../../foundation/ai/settings-mutations";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { AiRoutingDefaults } from "../../../shared/contracts/generated/AiRoutingDefaults";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import SettingsLoadFailure from "../components/SettingsLoadFailure.vue";
import AiRoutingSettings from "./models/components/AiRoutingSettings.vue";

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const aiStore = useAiSettingsStore();
const creationStore = useCreationLibraryStore();
const { settings, ready: aiReady, loadError } = storeToRefs(aiStore);
const { ready: creationReady, loadError: creationLoadError } = storeToRefs(creationStore);

const failure = computed(() =>
  loadError.value
    ? { source: "ai" as const, cause: loadError.value, canReset: true }
    : creationLoadError.value
      ? { source: "creation" as const, cause: creationLoadError.value, canReset: true }
      : undefined,
);
const ready = computed(() => aiReady.value && creationReady.value);

async function updateDefaultRoute(key: keyof AiRoutingDefaults, target: TaskTargetRef | null) {
  try {
    await setDefaultExecutionTarget(key, target);
  } catch (error) {
    toast.error(`${t("settings.general.saveFailed")}: ${getErrorMessage(error)}`);
  }
}

async function hydrateStores() {
  await Promise.all([aiStore.ensureLoaded(), creationStore.ensureLoaded()]);
}

async function reload() {
  await hydrateStores();
}

async function resetSection() {
  const confirmed = await confirmAction({
    title: t("settings.general.reset.action"),
    description: t("settings.general.reset.confirmation"),
    confirmLabel: t("settings.general.reset.action"),
    danger: true,
  });
  if (!confirmed) return;
  if (failure.value?.source === "creation") await creationStore.reset();
  else await resetAiSettings();
}

function handleRequestModelCreation() {
  void router.push({
    path: route.path,
    query: { ...route.query, pane: "aiProviders", setting: "settings-ai-providers" },
  });
}

onMounted(hydrateStores);
</script>

<template>
  <section data-testid="ai-routing-settings-pane" class="min-w-0">
    <SettingsLoadFailure
      v-if="failure"
      :message="getErrorMessage(failure.cause)"
      :can-reset="failure.canReset"
      @retry="reload"
      @reset="resetSection"
    />
    <div v-else-if="ready">
      <p class="mb-4 max-w-2xl text-body text-fg-secondary">
        {{ t("settings.routing.description") }}
      </p>
      <AiRoutingSettings
        :defaults="settings.defaults"
        :connections="settings.connections"
        :desktop-agents="settings.desktopAgents"
        @request-model-creation="handleRequestModelCreation"
        @update-route="updateDefaultRoute"
      />
    </div>
    <div v-else class="flex items-center gap-2 text-caption text-fg-secondary" role="status">
      <UiSpinner size="sm" />{{ t("common.loading") }}
    </div>
  </section>
</template>
