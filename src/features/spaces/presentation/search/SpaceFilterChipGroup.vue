<script setup lang="ts" generic="T extends string = string">
import { computed } from "vue";

export type FilterChipOption<V extends string = string> = V | { label: string; value: V };

const props = defineProps<{
  items: readonly (T | { label: string; value: T })[];
  selected?: readonly T[] | undefined;
}>();

const emit = defineEmits<{
  toggle: [value: T];
}>();

const normalizedItems = computed(() =>
  props.items.map((item) => (typeof item === "string" ? { label: item, value: item } : item)),
);
</script>

<template>
  <div class="flex flex-wrap gap-1">
    <button
      v-for="item in normalizedItems"
      :key="item.value"
      type="button"
      class="rounded-control px-2 py-1 text-micro"
      :class="
        selected?.includes(item.value)
          ? 'bg-selected text-foreground'
          : 'bg-control text-foreground-secondary'
      "
      @click="emit('toggle', item.value)"
    >
      {{ item.label }}
    </button>
  </div>
</template>
