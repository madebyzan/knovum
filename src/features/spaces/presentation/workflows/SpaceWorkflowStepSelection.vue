<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import type { WorkflowCaptureCandidate } from "../../workflows/captured-workflow-ipc";
import { captureActionLabels } from "./workflow-capture-labels";

type Step = WorkflowCaptureCandidate["steps"][number];
const props = defineProps<{ steps: readonly Step[]; modelValue: readonly string[] }>();
const emit = defineEmits<{ "update:modelValue": [value: string[]] }>();
const { t } = useI18n();

function label(step: Step) {
  const key = captureActionLabels[step.actionId];
  return key ? t(`spaces.creation.captureActions.${key}`) : step.actionId;
}

function hasDependents(id: string) {
  return props.steps.some(
    (step) =>
      props.modelValue.includes(step.id) &&
      step.inputs.some((input) => input.sourceType === "stepOutput" && input.stepId === id),
  );
}

function choose(id: string, selected: boolean) {
  emit(
    "update:modelValue",
    selected ? [...props.modelValue, id] : props.modelValue.filter((candidate) => candidate !== id),
  );
}
</script>

<template>
  <fieldset class="space-y-2">
    <legend class="text-caption font-semibold text-foreground">
      {{ t("spaces.creation.workflowSteps") }}
    </legend>
    <div v-for="step in steps" :key="step.id" class="flex items-center gap-2 text-caption">
      <UiCheckbox
        :label="t('spaces.creation.keepWorkflowStep', { action: label(step) })"
        :model-value="modelValue.includes(step.id)"
        :disabled="hasDependents(step.id)"
        @update:model-value="choose(step.id, $event)"
      />
      <span class="min-w-0 flex-1 truncate text-foreground">{{ label(step) }}</span>
      <span v-if="hasDependents(step.id)" class="text-foreground-tertiary">{{
        t("spaces.creation.workflowStepRequired")
      }}</span>
    </div>
  </fieldset>
</template>
