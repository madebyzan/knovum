<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { storeToRefs } from "pinia";
import { computed, nextTick, onMounted, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useAiSettingsStore } from "../../../../foundation/ai/ai-settings-store";
import { useCreationLibraryStore } from "../../../../foundation/ai/creation-library-store";
import { setDefaultExecutionTarget } from "../../../../foundation/ai/execution-settings";
import { loadProviderDescriptors } from "../../../../foundation/ai/providers";
import { resetAiSettings } from "../../../../foundation/ai/settings-mutations";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { AiRoutingDefaults } from "../../../../shared/contracts/generated/AiRoutingDefaults";
import type { RoutingPurpose } from "../../../../shared/contracts/generated/RoutingPurpose";
import type { TaskTargetRef } from "../../../../shared/contracts/generated/TaskTargetRef";
import { confirmAction } from "../../../../shared/ui/confirm-action";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import { useProtectedExecutionTargets } from "../../use-protected-execution-targets";
import AiConnectionsSettings from "../AiConnectionsSettings.vue";
import AiRoutingSettings from "../AiRoutingSettings.vue";
import LocalSpeechResourcesSettings from "../LocalSpeechResourcesSettings.vue";
import SettingsLoadFailure from "../SettingsLoadFailure.vue";

const PROVIDER_CATALOG_KEY = ["integrations", "provider-descriptors"] as const;

const { t } = useI18n();
const aiStore = useAiSettingsStore();
const creationStore = useCreationLibraryStore();
const { settings, ready: aiReady, loadError } = storeToRefs(aiStore);
const { ready: creationReady, loadError: creationLoadError } = storeToRefs(creationStore);
const catalog = useQuery({
  queryKey: PROVIDER_CATALOG_KEY,
  queryFn: loadProviderDescriptors,
  staleTime: 30_000,
});
const providers = computed(() => catalog.data.value ?? []);
const createModelRequest = shallowRef<{ purpose: RoutingPurpose }>();
const failure = computed(() =>
  loadError.value
    ? { source: "ai" as const, cause: loadError.value, canReset: true }
    : creationLoadError.value
      ? { source: "creation" as const, cause: creationLoadError.value, canReset: true }
      : catalog.error.value
        ? { source: "catalog" as const, cause: catalog.error.value, canReset: false }
        : undefined,
);
const ready = computed(() => aiReady.value && creationReady.value && !catalog.isPending.value);
const { protectedConnectionIds, protectedModelTargets } = useProtectedExecutionTargets();

async function updateDefaultRoute(key: keyof AiRoutingDefaults, target: TaskTargetRef | null) {
  try {
    await setDefaultExecutionTarget(key, target);
  } catch (error) {
    toast.error(`${t("settings.ai.integrationSaveFailed")}: ${getErrorMessage(error)}`);
  }
}

async function hydrateStores() {
  await Promise.all([aiStore.ensureLoaded(), creationStore.ensureLoaded()]);
}

async function reload() {
  await Promise.all([hydrateStores(), catalog.refetch()]);
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

onMounted(hydrateStores);

async function manageModels(purpose: RoutingPurpose) {
  const request = { purpose };
  createModelRequest.value = request;
  await nextTick();
  if (createModelRequest.value === request) createModelRequest.value = undefined;
}
</script>

<template>
  <section data-testid="ai-models-settings-pane" class="min-w-0">
    <SettingsLoadFailure
      v-if="failure"
      :message="getErrorMessage(failure.cause)"
      :can-reset="failure.canReset"
      @retry="reload"
      @reset="resetSection"
    />
    <div v-else-if="ready">
      <p class="mb-4 max-w-2xl text-body text-fg-secondary">{{ t("settings.ai.description") }}</p>
      <div class="grid gap-5">
        <AiConnectionsSettings
          :connections="settings.connections"
          :create-model-request="createModelRequest"
          :providers="providers"
          :protected-connection-ids="protectedConnectionIds"
          :protected-model-targets="protectedModelTargets"
        />
        <AiRoutingSettings
          :defaults="settings.defaults"
          :connections="settings.connections"
          :desktop-agents="settings.desktopAgents"
          @manage-models="manageModels"
          @update-route="updateDefaultRoute"
        />
      </div>
    </div>
    <div v-else class="flex items-center gap-2 text-caption text-fg-secondary" role="status">
      <UiSpinner size="sm" />{{ t("common.loading") }}
    </div>
    <LocalSpeechResourcesSettings />
  </section>
</template>
