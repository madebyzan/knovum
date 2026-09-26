<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { captureActionLabels } from "./workflow-capture-labels";

interface Field {
  stepId: string;
  actionId: string;
  key: string;
  type: string;
  options: { value: string; label: string }[];
}

defineProps<{
  fields: readonly Field[];
  parameters: Record<string, Record<string, unknown>>;
  text: Record<string, string>;
}>();
const emit = defineEmits<{
  updateText: [key: string, value: string];
  updateBoolean: [stepId: string, key: string, value: boolean];
}>();
const { t } = useI18n();

function actionLabel(id: string) {
  const label = captureActionLabels[id];
  return label ? t(`spaces.creation.workflow.captureActions.${label}`) : id;
}

function parameterLabel(key: string) {
  return t(`spaces.creation.workflow.parameters.${key}`);
}
</script>

<template>
  <div v-for="field in fields" :key="`${field.stepId}:${field.key}`" class="space-y-1 text-caption">
    <label
      v-if="field.type !== 'boolean'"
      :for="`workflow-parameter-${field.stepId}-${field.key}`"
      class="text-foreground-secondary"
    >
      {{ actionLabel(field.actionId) }} · {{ parameterLabel(field.key) }}
    </label>
    <span v-else class="text-foreground-secondary">{{ actionLabel(field.actionId) }}</span>
    <UiSelect
      v-if="field.options.length"
      :id="`workflow-parameter-${field.stepId}-${field.key}`"
      :model-value="text[`${field.stepId}:${field.key}`] ?? ''"
      :label="parameterLabel(field.key)"
      :options="field.options"
      size="sm"
      @update:model-value="emit('updateText', `${field.stepId}:${field.key}`, $event)"
    />
    <UiCheckbox
      v-else-if="field.type === 'boolean'"
      :label="parameterLabel(field.key)"
      :model-value="Boolean(parameters[field.stepId]?.[field.key])"
      @update:model-value="emit('updateBoolean', field.stepId, field.key, $event)"
    />
    <UiInput
      v-else
      :id="`workflow-parameter-${field.stepId}-${field.key}`"
      :model-value="text[`${field.stepId}:${field.key}`] ?? ''"
      :type="field.type === 'number' ? 'number' : 'text'"
      size="sm"
      @update:model-value="emit('updateText', `${field.stepId}:${field.key}`, $event)"
    />
  </div>
</template>
