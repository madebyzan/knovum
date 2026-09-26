<script setup lang="ts">
import { ask } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { AgentKind, DesktopAgentSettings } from "../../../../foundation/ai/agent-models";
import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useExecutionCatalogQuery } from "../../../../foundation/ai/execution-catalog-query";
import AgentInstallationRow from "../AgentInstallationRow.vue";
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
  const confirmed = await ask(t("settings.general.resetSectionConfirmation"), {
    title: t("settings.general.resetSection"),
    kind: "warning",
    okLabel: t("settings.general.resetSection"),
    cancelLabel: t("common.cancel"),
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
    <p class="max-w-2xl text-body text-foreground-secondary">
      {{ t("settings.ai.desktopAgentsDescription") }}
    </p>
    <SettingsSection id="settings-desktop-agents" tabindex="-1">
      <div
        v-if="catalog.installationError.value"
        class="bg-warning-subtle border-b border-warning/30 px-4 py-3"
      >
        <p class="text-caption text-warning">{{ String(catalog.installationError.value) }}</p>
      </div>
      <div class="divide-y divide-divider">
        <AgentInstallationRow
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
