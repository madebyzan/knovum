<script setup lang="ts">
import { AlertTriangle } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, onMounted, watch, type HTMLAttributes } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import UiButton from "../../shared/ui/UiButton.vue";
import UiSelect from "../../shared/ui/UiSelect.vue";
import {
  availableExecutionTargets,
  decodeExecutionTarget,
  defaultExecutionTarget,
  encodeExecutionTarget,
  type ExecutionTargetOption,
} from "./ai-execution";
import type { AiRoutingPurpose, ExecutionTarget } from "./ai-models";
import { useAiSettingsStore } from "./ai-settings-store";
import { useExecutionCatalogQuery } from "./execution-catalog-query";

const props = defineProps<{
  purpose: AiRoutingPurpose;
  disabled?: boolean | undefined;
  compact?: boolean | undefined;
  class?: HTMLAttributes["class"];
}>();
const target = defineModel<ExecutionTarget | undefined>();
const { t } = useI18n();
const router = useRouter();
const store = useAiSettingsStore();
const { settings, ready: settingsReady, loadError: settingsError } = storeToRefs(store);
const catalog = useExecutionCatalogQuery();
const agentWarning = computed(() => catalog.descriptorError.value ?? catalog.installationError.value);
const options = computed(() =>
  availableExecutionTargets(
    settings.value,
    props.purpose,
    catalog.data.value?.agents ?? [],
    catalog.data.value?.installations ?? [],
  ).map((option) =>
    Object.assign({}, option, {
      description: executionTargetDescription(option),
    }),
  ),
);
const ready = computed(() => settingsReady.value);
const loadError = computed(() => settingsError.value);
const value = computed(() => (target.value ? encodeExecutionTarget(target.value) : ""));

function executionTargetDescription(option: ExecutionTargetOption): string {
  if (option.executorType === "agent" && agentWarning.value) {
    return `${t("spaces.assistant.aiSettingsUnavailable")}: ${String(agentWarning.value)}`;
  }
  if (option.disabledReason) return option.disabledReason;
  if (option.statusDetail) return option.statusDetail;
  if (option.capabilitySource === "model-native") return t("settings.ai.executionTargetModelNative");
  return t(
    option.availability === "unverified"
      ? "settings.ai.executionTargetAgentWorkflow"
      : "settings.ai.executionTargetAgentReady",
  );
}

function isSelectableExecutionTarget(encodedTarget: string): boolean {
  return options.value.some(
    (option) => !option.disabled && encodeExecutionTarget(option.target) === encodedTarget,
  );
}

function currentTargetIsSelectable(): boolean {
  return target.value !== undefined && isSelectableExecutionTarget(encodeExecutionTarget(target.value));
}

function catalogIsPendingForAgent(candidate: ExecutionTarget | undefined): boolean {
  return candidate?.type === "agent" && !catalog.data.value;
}

function resolveConfiguredTarget(): ExecutionTarget | undefined {
  const configured = defaultExecutionTarget(settings.value, props.purpose) ?? undefined;
  if (catalogIsPendingForAgent(configured)) return configured;
  if (!configured) return;
  return isSelectableExecutionTarget(encodeExecutionTarget(configured)) ? configured : undefined;
}

function synchronizeTarget() {
  if (!ready.value || currentTargetIsSelectable() || catalogIsPendingForAgent(target.value)) return;
  target.value = resolveConfiguredTarget();
}

async function load() {
  await store.ensureLoaded();
  if (catalog.descriptorError.value) await catalog.refetchDescriptors();
}

watch([ready, options, () => props.purpose], synchronizeTarget, { immediate: true });

onMounted(load);
</script>

<template>
  <div :class="[props.class, compact && 'flex min-w-0 items-center gap-1']">
    <div
      v-if="loadError"
      :class="
        compact
          ? 'flex items-center gap-2 text-caption text-danger'
          : 'space-y-2 rounded-control border border-danger/30 bg-danger-subtle p-3'
      "
    >
      <p :class="compact ? 'truncate' : 'text-label font-medium'">
        {{ t("spaces.assistant.aiSettingsUnavailable") }}
      </p>
      <p v-if="!compact" class="text-meta wrap-break-word text-foreground-secondary">
        {{ String(loadError) }}
      </p>
      <UiButton size="sm" variant="secondary" @click="load">{{ t("common.retry") }}</UiButton>
    </div>
    <template v-else-if="compact">
      <UiSelect
        :model-value="value"
        :label="t('settings.ai.executionTarget')"
        :options="options"
        :disabled="disabled || !ready || Boolean(loadError) || options.length === 0"
        :placeholder="t('settings.ai.noCompatibleTarget')"
        size="sm"
        class="min-w-0 flex-1 border-transparent bg-transparent shadow-none hover:bg-control-hover"
        @update:model-value="target = decodeExecutionTarget($event)"
      />
      <AlertTriangle
        v-if="agentWarning"
        class="size-(--knovum-icon-size-xs) shrink-0 text-warning"
        :aria-label="`${t('spaces.assistant.aiSettingsUnavailable')}: ${String(agentWarning)}`"
      />
    </template>
    <div
      v-else-if="ready && options.length === 0"
      class="space-y-2 rounded-control border border-border/70 bg-panel/60 p-3"
    >
      <div class="space-y-0.5">
        <p class="text-label font-medium text-foreground">{{ t("spaces.assistant.aiDisabledTitle") }}</p>
        <p class="text-meta leading-relaxed text-foreground-secondary">
          {{ t("spaces.assistant.aiDisabledDescription") }}
        </p>
      </div>
      <div class="flex flex-wrap gap-2">
        <UiButton
          size="sm"
          variant="secondary"
          @click="router.push({ name: 'settings', query: { pane: 'aiModels' } })"
        >
          {{ t("spaces.assistant.openAiSettings") }}
        </UiButton>
        <UiButton
          size="sm"
          variant="secondary"
          @click="router.push({ name: 'settings', query: { pane: 'desktopAgents' } })"
        >
          {{ t("spaces.assistant.openAgentSettings") }}
        </UiButton>
      </div>
    </div>
    <label v-else class="block space-y-1.5 text-label font-medium">
      {{ t("settings.ai.executionTarget") }}
      <UiSelect
        :model-value="value"
        :label="t('settings.ai.executionTarget')"
        :options="options"
        :disabled="disabled || !ready || Boolean(loadError) || options.length === 0"
        :placeholder="t('settings.ai.noCompatibleTarget')"
        @update:model-value="target = decodeExecutionTarget($event)"
      />
      <span v-if="agentWarning" class="block text-caption font-normal text-warning">
        {{ t("spaces.assistant.aiSettingsUnavailable") }}: {{ String(agentWarning) }}
      </span>
    </label>
  </div>
</template>
