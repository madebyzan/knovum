<script setup lang="ts">
import UiNumberField from "../../../../../shared/ui/UiNumberField.vue";

/** One frame dimension, edited in place in an action bar: a quiet field led by its axis letter. */
const props = defineProps<{
  axis: "W" | "H";
  label: string;
  value: number;
  min: number;
  disabled?: boolean | undefined;
  hint?: string | undefined;
}>();
const emit = defineEmits<{ commit: [value: number] }>();
const INTEGER = { maximumFractionDigits: 0, useGrouping: false } as const;

// The field reports its shown value again on blur after Enter; only a different size is an edit.
function commit(value: number | undefined) {
  if (value !== undefined && value !== Math.round(props.value)) emit("commit", value);
}
</script>

<template>
  <label
    class="flex h-(--knovum-control-height-md) shrink-0 items-center rounded-md bg-surface-control pl-2 text-caption text-fg-tertiary transition-colors duration-100 ease-standard focus-within:bg-surface-raised focus-within:ring-1 focus-within:ring-stroke-strong focus-within:ring-inset hover:bg-surface-control-hover has-disabled:bg-surface-control has-disabled:opacity-40 motion-reduce:transition-none"
    :title="hint"
  >
    <span aria-hidden="true">{{ axis }}</span>
    <span class="sr-only">{{ label }}</span>
    <UiNumberField
      bare
      size="sm"
      :min="min"
      :disabled="disabled"
      :format-options="INTEGER"
      :model-value="Math.round(value)"
      class="h-full w-14 px-1.5 tabular-nums"
      @update:model-value="commit"
    />
  </label>
</template>
