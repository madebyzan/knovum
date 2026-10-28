<script setup lang="ts">
import UiNumberField from "../../../../../shared/ui/UiNumberField.vue";

/** One frame dimension, edited in place in an action bar: a quiet field led by its axis letter. */
defineProps<{
  axis: "W" | "H";
  label: string;
  value: number;
  min: number;
  disabled?: boolean;
  hint?: string;
}>();
const emit = defineEmits<{ commit: [value: number] }>();
const INTEGER = { maximumFractionDigits: 0, useGrouping: false } as const;
</script>

<template>
  <label
    class="flex h-(--knovum-control-height-md) shrink-0 items-center rounded-md bg-surface-control pl-2 text-caption text-fg-tertiary transition-colors duration-100 focus-within:bg-surface-raised focus-within:ring-2 focus-within:ring-stroke-focus hover:bg-surface-control-hover has-[:disabled]:opacity-50 has-[:disabled]:hover:bg-surface-control"
    :title="hint"
  >
    <span aria-hidden="true">{{ axis }}</span>
    <span class="sr-only">{{ label }}</span>
    <UiNumberField
      size="sm"
      :min="min"
      :disabled="disabled"
      :format-options="INTEGER"
      :model-value="Math.round(value)"
      class="h-full w-14 border-0 bg-transparent px-1.5 text-fg-primary tabular-nums hover:border-0 focus:border-0"
      @update:model-value="$event !== undefined && emit('commit', $event)"
    />
  </label>
</template>
