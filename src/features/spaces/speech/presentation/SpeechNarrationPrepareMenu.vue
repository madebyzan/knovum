<script setup lang="ts">
import { AudioLines, LoaderCircle, RotateCcw, Sparkles, Square } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import AiReadinessNotice from "../../../../foundation/ai/AiReadinessNotice.vue";
import type { NarrationPreparationMode } from "../../../../shared/contracts/generated/NarrationPreparationMode";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useNarrationTarget } from "../use-narration-target";

const props = defineProps<{
  disabled: boolean;
  empty: boolean;
  preparing: boolean;
  canRestore: boolean;
  compact?: boolean | undefined;
  label?: string | undefined;
  restoreLabel?: string | undefined;
  progressLabel?: string | undefined;
}>();
const emit = defineEmits<{
  prepare: [mode: NarrationPreparationMode, target?: TaskTarget];
  cancel: [];
  restore: [];
}>();
const { t } = useI18n();
const open = shallowRef(false);
const { options, selectedValue, selectedTarget, readiness, ready, selectValue } = useNarrationTarget();
const triggerDisabled = computed(() => props.disabled || props.empty);
const triggerLabel = computed(() => props.label ?? t("spaces.speech.project.prepareNarration.action"));
const modes = computed(
  () =>
    [
      {
        mode: "clean" as const,
        icon: Sparkles,
        label: t("spaces.speech.project.prepareNarration.clean"),
        description: t("spaces.speech.project.prepareNarration.cleanDescription"),
      },
      {
        mode: "adapt" as const,
        icon: AudioLines,
        label: t("spaces.speech.project.prepareNarration.adapt"),
        description: t("spaces.speech.project.prepareNarration.adaptDescription"),
      },
    ] as const,
);

function start(mode: NarrationPreparationMode) {
  if (!ready.value || triggerDisabled.value) return;
  open.value = false;
  emit("prepare", mode, selectedTarget.value);
}
</script>

<template>
  <div class="flex shrink-0 items-center gap-1">
    <template v-if="preparing">
      <UiButton size="sm" variant="ghost" @click="emit('cancel')">
        <LoaderCircle class="size-3 animate-spin motion-reduce:animate-none" />
        <Square class="size-2.5" />
        <span v-if="progressLabel" class="tabular-nums">{{ progressLabel }}</span>
        <span>{{ t("common.cancel") }}</span>
      </UiButton>
    </template>
    <template v-else>
      <UiIconButton
        v-if="canRestore"
        size="sm"
        tooltip
        :label="restoreLabel ?? t('spaces.speech.project.prepareNarration.restore')"
        :disabled="disabled"
        @click="emit('restore')"
      >
        <RotateCcw class="size-3" />
      </UiIconButton>
      <UiPopover v-model:open="open" align="end" class="w-72 space-y-2 p-2.5">
        <template #trigger>
          <UiIconButton v-if="compact" size="sm" tooltip :label="triggerLabel" :disabled="triggerDisabled">
            <UiKnovumAiIcon class="size-3.5" />
          </UiIconButton>
          <UiButton
            v-else
            size="sm"
            variant="ghost"
            :disabled="triggerDisabled"
            :title="t('spaces.speech.project.prepareNarration.hint')"
          >
            <UiKnovumAiIcon class="size-3.5" />
            <span>{{ triggerLabel }}</span>
          </UiButton>
        </template>
        <div class="space-y-1.5 border-b border-stroke-divider pb-2">
          <label class="block space-y-1 text-caption-sm font-medium text-fg-secondary">
            <span>{{ t("spaces.speech.project.prepareNarration.model") }}</span>
            <UiSelect
              size="sm"
              :model-value="selectedValue"
              :label="t('spaces.speech.project.prepareNarration.model')"
              :options="options"
              :placeholder="t('ai.target.none')"
              :disabled="options.length === 0"
              @update:model-value="selectValue"
            />
          </label>
          <AiReadinessNotice :readiness="readiness" />
        </div>
        <div class="space-y-1">
          <button
            v-for="item in modes"
            :key="item.mode"
            type="button"
            class="flex w-full items-start gap-2 rounded-md px-2 py-1.5 text-left transition-colors outline-none hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent"
            :disabled="!ready || triggerDisabled"
            @click="start(item.mode)"
          >
            <component :is="item.icon" class="mt-0.5 size-3.5 shrink-0 text-fg-secondary" />
            <span class="min-w-0">
              <span class="block text-caption font-medium text-fg-primary">{{ item.label }}</span>
              <span class="block text-caption-sm leading-snug text-fg-tertiary">{{ item.description }}</span>
            </span>
          </button>
        </div>
      </UiPopover>
    </template>
  </div>
</template>
