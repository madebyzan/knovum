<script setup lang="ts">
import { computed, onMounted, ref, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { listSpaceActions, type SpaceActionDefinition } from "../../workflows/action-catalog";
import { reviseCapturedWorkflow, type SavedCapturedWorkflow } from "../../workflows/captured-workflow-ipc";
import SpaceWorkflowOutputEditor from "./SpaceWorkflowOutputEditor.vue";
import SpaceWorkflowParameterFields from "./SpaceWorkflowParameterFields.vue";
import SpaceWorkflowStepSelection from "./SpaceWorkflowStepSelection.vue";
import { workflowParameterOptions } from "./workflow-parameter-options";

const props = defineProps<{ workflow: SavedCapturedWorkflow }>();
const emit = defineEmits<{ saved: [value: SavedCapturedWorkflow]; cancel: [] }>();
const { t } = useI18n();
const catalog = shallowRef<SpaceActionDefinition[]>([]);
const catalogError = ref("");
onMounted(async () => {
  try {
    catalog.value = await listSpaceActions();
  } catch (cause) {
    catalogError.value = String(cause);
  }
});
const outputSlots = ref(structuredClone(props.workflow.candidate.outputSlots));
const includedStepIds = ref(props.workflow.candidate.steps.map(({ id }) => id));
const steps = computed(() =>
  props.workflow.candidate.steps.filter(({ id }) => includedStepIds.value.includes(id)),
);
const parameterText = ref<Record<string, string>>({});
const parameters = ref(
  Object.fromEntries(
    props.workflow.candidate.steps.map((step) => [step.id, structuredClone(step.executionSpec.parameters)]),
  ),
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
    names.length > 0 &&
    names.every((name) => name.length > 0 && Array.from(name).length <= 120) &&
    new Set(names).size === names.length &&
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
  parameters.value[stepId] = { ...parameters.value[stepId], [key]: value };
}

function setText(key: string, value: string) {
  parameterText.value = { ...parameterText.value, [key]: value };
}

function setIncludedSteps(ids: string[]) {
  includedStepIds.value = ids;
  outputSlots.value = outputSlots.value.filter(({ stepId }) => ids.includes(stepId));
}

async function save() {
  if (!valid.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    const stepParameters: Record<string, Record<string, unknown>> = Object.fromEntries(
      steps.value.map(({ id }) => [id, structuredClone(parameters.value[id] ?? {})]),
    );
    for (const field of fields.value) {
      if (field.type === "boolean") continue;
      const text = parameterText.value[`${field.stepId}:${field.key}`] ?? "";
      stepParameters[field.stepId]![field.key] = field.type === "number" ? Number(text) : text;
    }
    emit(
      "saved",
      await reviseCapturedWorkflow({
        workflowId: props.workflow.workflowId,
        expectedVersion: props.workflow.version,
        outputSlots: outputSlots.value,
        stepParameters,
        removedStepIds: props.workflow.candidate.steps
          .filter(({ id }) => !includedStepIds.value.includes(id))
          .map(({ id }) => id),
      }),
    );
  } catch (cause) {
    error.value = String(cause);
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
