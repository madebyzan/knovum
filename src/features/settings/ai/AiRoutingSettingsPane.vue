<script setup lang="ts">
import { storeToRefs } from "pinia";
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { setDefaultExecutionTarget } from "../../../foundation/ai/execution-settings";
import { resetAiSettings } from "../../../foundation/ai/settings-mutations";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { AiRoutingDefaults } from "../../../shared/contracts/generated/AiRoutingDefaults";
import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import SettingsLoadFailure from "../components/SettingsLoadFailure.vue";
import AiRoutingSettings from "./models/components/AiRoutingSettings.vue";

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const aiStore = useAiSettingsStore();
const { settings, ready, loadError } = storeToRefs(aiStore);

async function updateDefaultRoute(key: keyof AiRoutingDefaults, target: TaskTarget | null) {
  try {
    await setDefaultExecutionTarget(key, target);
  } catch (error) {
    toast.error(`${t("settings.general.saveFailed")}: ${getErrorMessage(error)}`);
  }
}

async function reload() {
  await aiStore.ensureLoaded();
}

async function resetSection() {
  const confirmed = await confirmAction({
    title: t("settings.general.reset.action"),
    description: t("settings.general.reset.confirmation"),
    confirmLabel: t("settings.general.reset.action"),
    danger: true,
  });
  if (!confirmed) return;
  await resetAiSettings();
}

function handleRequestModelCreation() {
  void router.push({
    path: route.path,
    query: { ...route.query, pane: "aiProviders", setting: "settings-ai-providers" },
  });
}

onMounted(reload);
</script>

<template>
  <section data-testid="ai-routing-settings-pane" class="min-w-0">
    <SettingsLoadFailure
      v-if="loadError"
      :message="getErrorMessage(loadError)"
      can-reset
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
