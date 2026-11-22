<script setup lang="ts" generic="T extends string">
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
}>();

const emit = defineEmits<{ "update:modelValue": [value: T] }>();
</script>

<template>
  <div class="inline-flex shrink-0 items-center rounded-md bg-surface-control/90 p-0.5">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      :title="option.title ?? option.label"
      class="inline-flex items-center gap-1 rounded-[5px] text-caption-sm font-medium transition-[background-color,color,box-shadow] duration-120 ease-standard"
      :class="[
        iconOnly ? 'px-1.5 py-1' : 'px-2 py-0.5',
        modelValue === option.value
          ? 'bg-surface-raised text-fg-primary shadow-control'
          : 'text-fg-secondary hover:text-fg-primary',
      ]"
      @click="emit('update:modelValue', option.value)"
    >
      <component :is="option.icon" :class="iconOnly ? 'size-3.5' : 'size-3'" :stroke-width="1.65" />
      <span v-if="!iconOnly && option.label">{{ option.label }}</span>
    </button>
  </div>
</template>
