<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { disableDesktopAgent, enableDesktopAgent } from "../../../foundation/ai/execution-settings";
import { resetAiSettings } from "../../../foundation/ai/settings-mutations";
import { useExecutionCatalogQuery } from "../../../foundation/ai/use-execution-catalog-query";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { ExternalAgentKind } from "../../../shared/contracts/generated/ExternalAgentKind";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import SettingsLoadFailure from "../components/SettingsLoadFailure.vue";
import ExternalAgentAvailability from "./components/ExternalAgentAvailability.vue";
import DesktopAgentsSettingsAction from "./DesktopAgentsSettingsAction.vue";

const { t } = useI18n();
const aiStore = useAiSettingsStore();
const { settings, ready: aiReady, loadError: aiLoadError } = storeToRefs(aiStore);
const saving = shallowRef(false);
const {
  data: executionCatalog,
  installationFetching: inspecting,
  descriptorPending,
  descriptorError,
  installationError,
  refetchDescriptors,
} = useExecutionCatalogQuery({ detectInstallations: true });
const agents = computed(() => executionCatalog.value?.agents ?? []);
const installations = computed(() => executionCatalog.value?.installations ?? []);
const ready = computed(() => aiReady.value && !descriptorPending.value);
const failure = computed(() =>
  aiLoadError.value
    ? { source: "ai" as const, cause: aiLoadError.value, canReset: true }
    : descriptorError.value
      ? { source: "catalog" as const, cause: descriptorError.value, canReset: false }
      : undefined,
);

function findInstallation(kind: ExternalAgentKind) {
  return installations.value.find((agentInstallation) => agentInstallation.kind === kind);
}
function isEnabled(kind: ExternalAgentKind) {
  return settings.value.desktopAgents.some((agent) => agent.kind === kind);
}

async function setEnabled(kind: ExternalAgentKind, enabled: boolean) {
  if (!ready.value || saving.value) return;
  saving.value = true;
  try {
    if (enabled) await enableDesktopAgent(kind);
    else await disableDesktopAgent(kind);
  } catch (error) {
    toast.error(`${t("settings.general.saveFailed")}: ${getErrorMessage(error)}`);
  } finally {
    saving.value = false;
  }
}

async function load() {
  await aiStore.ensureLoaded();
  if (descriptorError.value) await refetchDescriptors();
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

onMounted(load);
</script>

<template>
  <div>
    <DesktopAgentsSettingsAction />
    <SettingsLoadFailure
      v-if="failure"
      :message="getErrorMessage(failure.cause)"
      :can-reset="failure.canReset"
      @retry="load"
      @reset="resetSection"
    />
    <div
      v-else
      class="grid gap-5"
      :class="(!ready || saving) && 'pointer-events-none opacity-60'"
      :inert="!ready || saving"
    >
      <p class="max-w-2xl text-body text-fg-secondary">
        {{ t("settings.agents.description") }}
      </p>
      <UiSettingsSection id="settings-desktop-agents" tabindex="-1">
        <div
          v-if="installationError"
          class="border-b border-stroke-warning/30 bg-surface-warning-subtle px-4 py-3"
        >
          <p class="text-caption text-fg-warning">{{ getErrorMessage(installationError) }}</p>
        </div>
        <div class="divide-y divide-stroke-divider">
          <ExternalAgentAvailability
            v-for="agent in agents"
            :key="agent.kind"
            :agent="agent"
            :installation="findInstallation(agent.kind)"
            :enabled="isEnabled(agent.kind)"
            :checking="inspecting"
            :check-failed="Boolean(installationError)"
            @toggle="setEnabled(agent.kind, $event)"
          />
        </div>
      </UiSettingsSection>
    </div>
  </div>
</template>
