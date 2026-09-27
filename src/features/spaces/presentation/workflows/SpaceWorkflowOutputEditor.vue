<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { WorkflowCaptureCandidate } from "../../workflows/captured-workflow-ipc";
import { captureActionLabel } from "./workflow-capture-labels";

type Step = WorkflowCaptureCandidate["steps"][number];
type Output = WorkflowCaptureCandidate["outputSlots"][number];
const props = defineProps<{ steps: readonly Step[]; modelValue: readonly Output[] }>();
const emit = defineEmits<{ "update:modelValue": [value: Output[]] }>();
const { t } = useI18n();

function outputFor(stepId: string) {
  return props.modelValue.find((output) => output.stepId === stepId);
}

function label(step: Step) {
  return captureActionLabel(step.actionId, t);
}

function choose(step: Step, selected: boolean) {
  if (!selected) {
    emit(
      "update:modelValue",
      props.modelValue.filter((output) => output.stepId !== step.id),
    );
    return;
  }
  if (outputFor(step.id)) return;
  emit("update:modelValue", [
    ...props.modelValue,
    {
      id: crypto.randomUUID(),
      name: label(step),
      kind: step.outputKind,
      stepId: step.id,
    },
  ]);
}

function rename(stepId: string, name: string) {
  emit(
    "update:modelValue",
    props.modelValue.map((output) =>
      output.stepId === stepId ? Object.assign({}, output, { name }) : output,
    ),
  );
}
</script>

<template>
  <fieldset class="space-y-2">
    <legend class="text-caption font-semibold text-fg-primary">
      {{ t("spaces.creation.workflow.outputs.title") }}
    </legend>
    <div v-for="step in steps" :key="step.id" class="space-y-1 rounded-md border border-stroke-divider p-2">
      <div class="flex items-center gap-2 text-caption text-fg-primary">
        <UiCheckbox
          :label="t('spaces.creation.workflow.outputs.select', { action: label(step) })"
          :model-value="Boolean(outputFor(step.id))"
          @update:model-value="choose(step, $event)"
        />
        <span>{{ label(step) }}</span>
      </div>
      <UiInput
        v-if="outputFor(step.id)"
        :model-value="outputFor(step.id)?.name ?? ''"
        :aria-label="t('spaces.creation.workflow.outputs.name', { index: steps.indexOf(step) + 1 })"
        size="sm"
        @update:model-value="rename(step.id, $event)"
      />
    </div>
  </fieldset>
</template>
