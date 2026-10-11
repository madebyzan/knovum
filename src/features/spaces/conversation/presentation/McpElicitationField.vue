<script setup lang="ts">
import { computed } from "vue";

import {
  elicitationOptions,
  type McpElicitationField,
  type McpElicitationValue,
} from "../mcp-elicitation-form";
const value = defineModel<McpElicitationValue | undefined>();
const props = defineProps<{ name: string; field: McpElicitationField; required: boolean }>();
const options = computed(() => elicitationOptions(props.field));
const inputType = computed(() =>
  props.field.type === "string"
    ? ({ email: "email", uri: "url", date: "date", "date-time": "text" }[props.field.format ?? "date-time"] ??
      "text")
    : "number",
);
</script>

<template>
  <label class="block text-caption"
    >{{ field.title ?? name }}
    <select
      v-if="options"
      v-model="value"
      :multiple="field.type === 'array'"
      class="ml-2 rounded-sm bg-surface-control p-1"
      :required="required"
    >
      <option v-if="field.type !== 'array'" value="" />
      <option v-for="option in options" :key="option.value" :value="option.value">{{ option.label }}</option>
    </select>
    <input v-else-if="field.type === 'boolean'" v-model="value" type="checkbox" class="ml-2" />
    <input
      v-else
      v-model="value"
      :type="inputType"
      :step="field.type === 'integer' ? 1 : 'any'"
      :min="field.type === 'number' || field.type === 'integer' ? field.minimum : undefined"
      :max="field.type === 'number' || field.type === 'integer' ? field.maximum : undefined"
      :minlength="field.type === 'string' ? field.minLength : undefined"
      :maxlength="field.type === 'string' ? field.maxLength : undefined"
      :required="required"
      class="block w-full rounded-sm border border-stroke-divider bg-surface-control px-2 py-1"
    />
    <span v-if="field.description" class="block text-fg-secondary">{{ field.description }}</span>
  </label>
</template>
