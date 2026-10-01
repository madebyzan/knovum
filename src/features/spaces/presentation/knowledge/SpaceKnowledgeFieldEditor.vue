<script setup lang="ts">
import { isNumber, isString } from "es-toolkit";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiDateField from "../../../../shared/ui/UiDateField.vue";
import UiNumberField from "../../../../shared/ui/UiNumberField.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { SpaceObjectFieldDefinition } from "../../knowledge/model/object-types";
import type { SpaceKnowledgeObjectPropertyValue } from "../../model/nodes/node-payload";

const modelValue = defineModel<SpaceKnowledgeObjectPropertyValue>();
const props = defineProps<{
  field: SpaceObjectFieldDefinition;
  nodeOptions: readonly { value: string; label: string }[];
}>();
const { t } = useI18n();

const stringValue = computed({
  get: () => (isString(modelValue.value) ? modelValue.value : ""),
  set: (value: string) => {
    modelValue.value = value || undefined;
  },
});
const numberValue = computed<number | undefined>({
  get: () => (isNumber(modelValue.value) ? modelValue.value : undefined),
  set: (value) => {
    modelValue.value = value;
  },
});
const booleanValue = computed<boolean>({
  get: () => modelValue.value === true,
  set: (value) => {
    modelValue.value = value;
  },
});
const selectOptions = computed(() => {
  if (props.field.kind === "enum")
    return (props.field.options ?? []).map((value) => ({ value, label: value }));
  return props.nodeOptions;
});
const selectValue = computed({
  get: () => (isString(modelValue.value) && modelValue.value ? modelValue.value : "__none__"),
  set: (value: string) => {
    modelValue.value = value === "__none__" ? undefined : value;
  },
});
const choices = computed(() =>
  props.field.required
    ? selectOptions.value
    : [{ value: "__none__", label: t("spaces.inspector.none") }, ...selectOptions.value],
);
</script>

<template>
  <label class="block">
    <span class="mb-1 block text-caption-sm text-fg-tertiary"
      >{{ field.label }}<span v-if="field.required"> *</span></span
    >
    <UiNumberField v-if="field.kind === 'number'" v-model="numberValue" />
    <label
      v-else-if="field.kind === 'boolean'"
      class="flex h-(--knovum-control-height-md) items-center gap-2 text-label text-fg-secondary"
    >
      <UiCheckbox v-model="booleanValue" :label="field.label" />
      {{ field.label }}
    </label>
    <UiSelect
      v-else-if="field.kind === 'enum' || field.kind === 'node-ref'"
      v-model="selectValue"
      :label="field.label"
      :options="choices"
    />
    <UiDateField v-else-if="field.kind === 'date'" v-model="stringValue" />
    <UiTextarea v-else v-model="stringValue" :rows="3" />
  </label>
</template>
