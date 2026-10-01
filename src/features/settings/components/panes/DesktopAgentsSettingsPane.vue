<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useExecutionCatalogQuery } from "../../../../foundation/ai/execution-catalog-query";
import type { AgentKind } from "../../../../shared/contracts/generated/AgentKind";
import type { DesktopAgentSettings } from "../../../../shared/contracts/generated/DesktopAgentSettings";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import AgentAvailability from "../AgentAvailability.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";
import SettingsSection from "../SettingsSection.vue";

const { t } = useI18n();
const aiStore = useAiSettingsStore();
const { settings, ready: aiReady, loadError: aiLoadError, saveError } = storeToRefs(aiStore);
const catalog = useExecutionCatalogQuery({ detectInstallations: true });
const agents = computed(() => catalog.data.value?.agents ?? []);
const installations = computed(() => catalog.data.value?.installations ?? []);
const inspecting = computed(() => catalog.installationFetching.value);
const ready = computed(() => aiReady.value && !catalog.descriptorPending.value);
const failure = computed(() =>
  aiLoadError.value
    ? { source: "ai" as const, cause: aiLoadError.value, canReset: true }
    : catalog.descriptorError.value
      ? { source: "catalog" as const, cause: catalog.descriptorError.value, canReset: false }
      : undefined,
);

const installation = (kind: AgentKind) =>
  installations.value.find((agentInstallation) => agentInstallation.kind === kind);
const isEnabled = (kind: AgentKind) => settings.value.desktopAgents.some((agent) => agent.kind === kind);

watch(saveError, (error) => {
  if (error) toast.error(`${t("settings.ai.integrationSaveFailed")}: ${String(error)}`);
});

function setEnabled(kind: AgentKind, enabled: boolean) {
  const index = settings.value.desktopAgents.findIndex((agent) => agent.kind === kind);
  if (enabled && index === -1) {
    const agent: DesktopAgentSettings = { kind, enabled: true };
    settings.value.desktopAgents.push(agent);
  } else if (!enabled && index >= 0) {
    settings.value.desktopAgents.splice(index, 1);
  }
}

async function load() {
  await aiStore.ensureLoaded();
  if (catalog.descriptorError.value) await catalog.refetchDescriptors();
}

async function resetSection() {
  const confirmed = await confirmAction({
    title: t("settings.general.reset.action"),
    description: t("settings.general.reset.confirmation"),
    confirmLabel: t("settings.general.reset.action"),
    danger: true,
  });
  if (!confirmed) return;
  await aiStore.reset();
}

onMounted(load);
</script>

<template>
  <SettingsLoadFailure
    v-if="failure"
    :message="String(failure.cause)"
    :can-reset="failure.canReset"
    @retry="load"
    @reset="resetSection"
  />
  <div v-else class="grid gap-5" :class="!ready && 'pointer-events-none opacity-60'">
    <p class="max-w-2xl text-body text-fg-secondary">
      {{ t("settings.agents.description") }}
    </p>
    <SettingsSection id="settings-desktop-agents" tabindex="-1">
      <div
        v-if="catalog.installationError.value"
        class="border-b border-stroke-warning/30 bg-surface-warning-subtle px-4 py-3"
      >
        <p class="text-caption text-fg-warning">{{ String(catalog.installationError.value) }}</p>
      </div>
      <div class="divide-y divide-stroke-divider">
        <AgentAvailability
          v-for="agent in agents"
          :key="agent.kind"
          :agent="agent"
          :installation="installation(agent.kind)"
          :enabled="isEnabled(agent.kind)"
          :checking="inspecting"
          :check-failed="Boolean(catalog.installationError.value)"
          @toggle="setEnabled(agent.kind, $event)"
        />
      </div>
    </SettingsSection>
  </div>
</template>
