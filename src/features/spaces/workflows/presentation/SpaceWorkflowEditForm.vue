<script setup lang="ts">
import { cloneDeep } from "es-toolkit";
import { computed, onMounted, ref, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SavedCapturedWorkflow } from "../../contracts/generated/SavedCapturedWorkflow";
import type { SpaceActionDefinition } from "../../contracts/generated/SpaceActionDefinition";
import { listSpaceActions } from "../action-catalog";
import { reviseCapturedWorkflow } from "../captured-workflow-ipc";
import { workflowOutputNamesValid } from "../output-name-policy";
import SpaceWorkflowOutputEditor from "./SpaceWorkflowOutputEditor.vue";
import SpaceWorkflowParameterFields from "./SpaceWorkflowParameterFields.vue";
import SpaceWorkflowStepSelection from "./SpaceWorkflowStepSelection.vue";
import { workflowParameterOptions } from "./workflow-parameter-options";

const props = defineProps<{ workflow: SavedCapturedWorkflow }>();
const emit = defineEmits<{ saved: [value: SavedCapturedWorkflow]; cancel: [] }>();
const { t } = useI18n();
const workflow = cloneDeep(props.workflow);
const catalog = shallowRef<SpaceActionDefinition[]>([]);
const catalogError = ref("");
onMounted(async () => {
  try {
    catalog.value = await listSpaceActions();
  } catch (cause) {
    catalogError.value = getErrorMessage(cause);
  }
});
const outputSlots = ref(workflow.candidate.outputSlots);
const includedStepIds = ref(workflow.candidate.steps.map(({ id }) => id));
const steps = computed(() => workflow.candidate.steps.filter(({ id }) => includedStepIds.value.includes(id)));
const parameterText = ref<Record<string, string>>({});
const parameters = ref(
  Object.fromEntries(workflow.candidate.steps.map((step) => [step.id, step.executionSpec.parameters])),
);
const fields = computed(() =>
  steps.value.flatMap((step) =>
    Object.entries(step.executionSpec.parameters).flatMap(([key, value]) =>
      ["string", "number", "boolean"].includes(typeof value)
        ? [
            {
              stepId: step.id,
              actionId: step.actionId,
              key,
              type: typeof value,
              value,
              options: workflowParameterOptions(
                catalog.value.find((action) => action.id === step.actionId),
                key,
              ),
            },
          ]
        : [],
    ),
  ),
);
for (const field of fields.value) {
  if (field.type !== "boolean") parameterText.value[`${field.stepId}:${field.key}`] = String(field.value);
}
const valid = computed(() => {
  const names = outputSlots.value.map(({ name }) => name.trim());
  return (
    catalog.value.length > 0 &&
    steps.value.length > 0 &&
    workflowOutputNamesValid(names) &&
    fields.value.every(
      (field) =>
        field.type !== "number" ||
        (parameterText.value[`${field.stepId}:${field.key}`]?.trim() &&
          Number.isFinite(Number(parameterText.value[`${field.stepId}:${field.key}`]))),
    )
  );
});
const busy = ref(false);
const error = ref("");

function setBoolean(stepId: string, key: string, value: boolean) {
  parameters.value[stepId]![key] = value;
}

function setText(key: string, value: string) {
  parameterText.value = { ...parameterText.value, [key]: value };
}

function setIncludedSteps(stepIds: string[]) {
  includedStepIds.value = stepIds;
  outputSlots.value = outputSlots.value.filter(({ stepId }) => stepIds.includes(stepId));
}

async function save() {
  if (!valid.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    const stepParameters: Record<string, Record<string, unknown>> = Object.fromEntries(
      steps.value.map(({ id }) => [id, cloneDeep(parameters.value[id]!)]),
    );
    for (const field of fields.value) {
      if (field.type === "boolean") continue;
      const text = parameterText.value[`${field.stepId}:${field.key}`] ?? "";
      stepParameters[field.stepId]![field.key] = field.type === "number" ? Number(text) : text;
    }
    emit(
      "saved",
      await reviseCapturedWorkflow({
        workflowId: workflow.workflowId,
        expectedVersion: workflow.version,
        outputSlots: cloneDeep(outputSlots.value),
        stepParameters,
        removedStepIds: workflow.candidate.steps
          .filter(({ id }) => !includedStepIds.value.includes(id))
          .map(({ id }) => id),
      }),
    );
  } catch (cause) {
    error.value = getErrorMessage(cause);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <form class="space-y-3 border-t border-stroke-divider pt-3" @submit.prevent="save">
    <h3 class="text-caption font-semibold text-fg-primary">
      {{ t("spaces.creation.workflow.edit") }}
    </h3>
    <SpaceWorkflowStepSelection
      :steps="workflow.candidate.steps"
      :model-value="includedStepIds"
      @update:model-value="setIncludedSteps"
    />
    <SpaceWorkflowOutputEditor v-model="outputSlots" :steps="steps" />
    <p v-if="!catalog.length && !catalogError" class="text-caption text-fg-secondary" role="status">
      {{ t("spaces.creation.workflow.derivation.loading") }}
    </p>
    <SpaceWorkflowParameterFields
      :fields="catalog.length ? fields : []"
      :parameters="parameters"
      :text="parameterText"
      @update-text="setText"
      @update-boolean="setBoolean"
    />
    <p v-if="catalogError || error" role="alert" class="text-caption text-fg-danger">
      {{ catalogError || error }}
    </p>
    <div class="flex gap-2">
      <UiButton type="submit" size="sm" variant="primary" :disabled="!valid || busy">{{
        t("common.save")
      }}</UiButton>
      <UiButton type="button" size="sm" variant="ghost" :disabled="busy" @click="emit('cancel')">{{
        t("common.cancel")
      }}</UiButton>
    </div>
  </form>
</template>
