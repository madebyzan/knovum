<script setup lang="ts" generic="T extends string = string">
import { computed } from "vue";

export type FilterChipOption<V extends string = string> = V | { label: string; value: V };

const props = defineProps<{
  options: readonly (T | { label: string; value: T })[];
  selected?: readonly T[] | undefined;
}>();

const emit = defineEmits<{
  toggle: [value: T];
}>();

const normalizedOptions = computed(() =>
  props.options.map((option) => (typeof option === "string" ? { label: option, value: option } : option)),
);
</script>

<template>
  <div class="flex flex-wrap gap-1">
    <button
      v-for="option in normalizedOptions"
      :key="option.value"
      type="button"
      class="rounded-control px-2 py-1 text-micro"
      :class="
        selected?.includes(option.value)
          ? 'bg-selected text-foreground'
          : 'bg-control text-foreground-secondary'
      "
      @click="emit('toggle', option.value)"
    >
      {{ option.label }}
    </button>
  </div>
</template>
