<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, onMounted, watch, type HTMLAttributes } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import UiButton from "../../shared/ui/UiButton.vue";
import UiSelect from "../../shared/ui/UiSelect.vue";
import { availableExecutionTargets, defaultExecutionTarget } from "./ai-execution";
import { useAiSettingsStore } from "./ai-settings-store";
import { decodeExecutionTarget, encodeExecutionTarget } from "./execution-target-codec";
import type { AiRoutingPurpose, ExecutionTarget } from "./schema/models";

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
const options = computed(() =>
  availableExecutionTargets(settings.value, props.purpose).map((option) =>
    Object.assign({}, option, {
      description: t("ai.targetModelNative"),
    }),
  ),
);
const ready = computed(() => settingsReady.value);
const loadError = computed(() => settingsError.value);
const value = computed(() => (target.value ? encodeExecutionTarget(target.value) : ""));

function isSelectableExecutionTarget(encodedTarget: string): boolean {
  return options.value.some((option) => encodeExecutionTarget(option.target) === encodedTarget);
}

function currentTargetIsSelectable(): boolean {
  return target.value !== undefined && isSelectableExecutionTarget(encodeExecutionTarget(target.value));
}

function resolveConfiguredTarget(): ExecutionTarget | undefined {
  const configured = defaultExecutionTarget(settings.value, props.purpose) ?? undefined;
  if (!configured) return;
  return isSelectableExecutionTarget(encodeExecutionTarget(configured)) ? configured : undefined;
}

function synchronizeTarget() {
  if (!ready.value || currentTargetIsSelectable()) return;
  target.value = resolveConfiguredTarget();
}

async function load() {
  await store.ensureLoaded();
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
          ? 'flex items-center gap-2 text-caption-md text-fg-danger'
          : 'space-y-2 rounded-md border border-stroke-danger/30 bg-surface-danger-subtle p-3'
      "
    >
      <p :class="compact ? 'truncate' : 'text-label-md font-medium'">
        {{ t("ai.settingsUnavailable") }}
      </p>
      <p v-if="!compact" class="text-caption-md wrap-break-word text-fg-secondary">
        {{ String(loadError) }}
      </p>
      <UiButton size="sm" variant="secondary" @click="load">{{ t("common.retry") }}</UiButton>
    </div>
    <template v-else-if="compact">
      <UiSelect
        :model-value="value"
        :label="t('ai.target')"
        :options="options"
        :disabled="disabled || !ready || Boolean(loadError) || options.length === 0"
        :placeholder="t('ai.noCompatibleTarget')"
        size="sm"
        class="min-w-0 flex-1 border-transparent bg-transparent shadow-none hover:bg-surface-control-hover"
        @update:model-value="target = decodeExecutionTarget($event)"
      />
    </template>
    <div
      v-else-if="ready && options.length === 0"
      class="space-y-2 rounded-md border border-stroke-default/70 bg-surface-panel/60 p-3"
    >
      <div class="space-y-0.5">
        <p class="text-label-md font-medium text-fg-primary">{{ t("ai.disabledTitle") }}</p>
        <p class="text-caption-md leading-relaxed text-fg-secondary">
          {{ t("ai.disabledDescription") }}
        </p>
      </div>
      <div class="flex flex-wrap gap-2">
        <UiButton
          size="sm"
          variant="secondary"
          @click="router.push({ name: 'settings', query: { pane: 'aiModels' } })"
        >
          {{ t("ai.openSettings") }}
        </UiButton>
      </div>
    </div>
    <label v-else class="block space-y-1.5 text-label-md font-medium">
      {{ t("ai.target") }}
      <UiSelect
        :model-value="value"
        :label="t('ai.target')"
        :options="options"
        :disabled="disabled || !ready || Boolean(loadError) || options.length === 0"
        :placeholder="t('ai.noCompatibleTarget')"
        @update:model-value="target = decodeExecutionTarget($event)"
      />
    </label>
  </div>
</template>
