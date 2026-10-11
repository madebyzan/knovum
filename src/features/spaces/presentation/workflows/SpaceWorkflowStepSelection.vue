<script setup lang="ts">
import { without } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import type { CaptureCandidate } from "../../contracts/generated/CaptureCandidate";
import { captureActionLabel } from "./workflow-capture-labels";

type Step = CaptureCandidate["steps"][number];
const props = defineProps<{ steps: readonly Step[]; modelValue: readonly string[] }>();
const emit = defineEmits<{ "update:modelValue": [value: string[]] }>();
const { t } = useI18n();

const dependentStepIds = computed(() => {
  const selected = new Set(props.modelValue);
  const ids = new Set<string>();
  for (const step of props.steps) {
    if (!selected.has(step.id)) continue;
    for (const input of step.inputs) {
      if (input.sourceType === "stepOutput") ids.add(input.stepId);
    }
  }
  return ids;
});

function label(step: Step) {
  return captureActionLabel(step.actionId);
}

function choose(stepId: string, selected: boolean) {
  emit("update:modelValue", selected ? [...props.modelValue, stepId] : without(props.modelValue, stepId));
}
</script>

<template>
  <fieldset class="space-y-2">
    <legend class="text-caption font-semibold text-fg-primary">
      {{ t("spaces.creation.workflow.steps.title") }}
    </legend>
    <div v-for="step in steps" :key="step.id" class="flex items-center gap-2 text-caption">
      <UiCheckbox
        :label="t('spaces.creation.workflow.steps.keep', { action: label(step) })"
        :model-value="modelValue.includes(step.id)"
        :disabled="dependentStepIds.has(step.id)"
        @update:model-value="choose(step.id, $event)"
      />
      <span class="min-w-0 flex-1 truncate text-fg-primary">{{ label(step) }}</span>
      <span v-if="dependentStepIds.has(step.id)" class="text-fg-tertiary">{{
        t("spaces.creation.workflow.steps.required")
      }}</span>
    </div>
  </fieldset>
</template>
