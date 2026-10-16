<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { storeToRefs } from "pinia";
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";

import { useAiSettingsStore } from "../../../foundation/ai/ai-settings-store";
import { loadProviderDescriptors } from "../../../foundation/ai/providers";
import { resetAiSettings } from "../../../foundation/ai/settings-mutations";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { confirmAction } from "../../../shared/ui/confirm-action";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import SettingsLoadFailure from "../components/SettingsLoadFailure.vue";
import AiConnectionsSettings from "./connections/components/AiConnectionsSettings.vue";
import { useProtectedExecutionTargets } from "./use-protected-execution-targets";

const PROVIDER_CATALOG_KEY = ["integrations", "provider-descriptors"] as const;

const { t } = useI18n();
const aiStore = useAiSettingsStore();
const { settings, ready: aiReady, loadError } = storeToRefs(aiStore);
const catalog = useQuery({
  queryKey: PROVIDER_CATALOG_KEY,
  queryFn: loadProviderDescriptors,
  staleTime: 30_000,
});
const providers = computed(() => catalog.data.value ?? []);
const failure = computed(() =>
  loadError.value
    ? { cause: loadError.value, canReset: true }
    : catalog.error.value
      ? { cause: catalog.error.value, canReset: false }
      : undefined,
);
const ready = computed(() => aiReady.value && !catalog.isPending.value);
const { protectedConnectionIds, protectedModelTargets } = useProtectedExecutionTargets();

async function reload() {
  await Promise.all([aiStore.ensureLoaded(), catalog.refetch()]);
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

onMounted(reload);
</script>

<template>
  <section data-testid="ai-providers-settings-pane" class="min-w-0">
    <SettingsLoadFailure
      v-if="failure"
      :message="getErrorMessage(failure.cause)"
      :can-reset="failure.canReset"
      @retry="reload"
      @reset="resetSection"
    />
    <div v-else-if="ready">
      <p class="mb-4 max-w-2xl text-body text-fg-secondary">{{ t("settings.ai.overview.description") }}</p>
      <AiConnectionsSettings
        :connections="settings.connections"
        :providers="providers"
        :protected-connection-ids="protectedConnectionIds"
        :protected-model-targets="protectedModelTargets"
      />
    </div>
    <div v-else class="flex items-center gap-2 text-caption text-fg-secondary" role="status">
      <UiSpinner size="sm" />{{ t("common.loading") }}
    </div>
  </section>
</template>
