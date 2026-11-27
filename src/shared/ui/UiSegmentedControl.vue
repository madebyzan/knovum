<script setup lang="ts" generic="T extends string">
import { ToggleGroupItem, ToggleGroupRoot } from "reka-ui";
import type { Component } from "vue";

interface SegmentedOption<V extends string = string> {
  value: V;
  label?: string | undefined;
  title?: string | undefined;
  icon: Component;
}

defineProps<{
  modelValue: T;
  options: readonly SegmentedOption<T>[];
  iconOnly?: boolean;
  ariaLabel?: string | undefined;
}>();

const emit = defineEmits<{ "update:modelValue": [value: T] }>();

function handleUpdate(value: unknown) {
  if (typeof value === "string" && value.length > 0) emit("update:modelValue", value as T);
}
</script>

<template>
  <ToggleGroupRoot
    type="single"
    :model-value="modelValue"
    :aria-label="ariaLabel"
    class="inline-flex shrink-0 items-center rounded-md bg-surface-control/90 p-0.5"
    @update:model-value="handleUpdate"
  >
    <ToggleGroupItem
      v-for="option in options"
      :key="option.value"
      :value="option.value"
      :title="option.title ?? option.label"
      :aria-label="option.title ?? option.label"
      class="inline-flex items-center gap-1 rounded-[5px] text-caption-sm font-medium transition-[background-color,color,box-shadow] duration-120 ease-standard"
      :class="[
        iconOnly ? 'px-1.5 py-1' : 'px-2 py-0.5',
        modelValue === option.value
          ? 'bg-surface-raised text-fg-primary shadow-control'
          : 'text-fg-secondary hover:text-fg-primary',
      ]"
    >
      <component :is="option.icon" :class="iconOnly ? 'size-3.5' : 'size-3'" :stroke-width="1.65" />
      <span v-if="!iconOnly && option.label">{{ option.label }}</span>
    </ToggleGroupItem>
  </ToggleGroupRoot>
</template>
