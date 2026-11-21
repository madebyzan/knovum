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
  <div class="inline-flex shrink-0 rounded-md border border-stroke-divider bg-surface-control p-0.5">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      :title="option.title ?? option.label"
      class="inline-flex items-center gap-1 rounded-sm text-caption-sm font-medium transition-colors"
      :class="[
        iconOnly ? 'p-1' : 'px-2 py-0.5',
        modelValue === option.value ? 'bg-surface-raised text-fg-primary shadow-2xs' : 'text-fg-secondary',
      ]"
      @click="emit('update:modelValue', option.value)"
    >
      <component :is="option.icon" :class="iconOnly ? 'size-3.5' : 'size-3'" />
      <span v-if="!iconOnly && option.label">{{ option.label }}</span>
    </button>
  </div>
</template>
