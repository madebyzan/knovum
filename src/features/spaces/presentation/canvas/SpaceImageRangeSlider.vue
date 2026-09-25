<script setup lang="ts">
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from "reka-ui";

defineProps<{
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  minimal?: boolean;
  displayValue?: string;
}>();
const emit = defineEmits<{ update: [value: number] }>();
</script>

<template>
  <label class="grid gap-0.5 text-label text-foreground">
    <span v-if="!minimal" class="flex items-center justify-between">
      <span>{{ label }}</span>
      <span class="text-foreground-secondary tabular-nums">{{ displayValue ?? value }}</span>
    </span>
    <span v-else class="sr-only">{{ label }}</span>
    <SliderRoot
      :model-value="[value]"
      :min="min"
      :max="max"
      :step="step ?? 1"
      :aria-label="label"
      class="relative flex h-4 w-full touch-none items-center select-none"
      @update:model-value="(next) => next?.[0] !== undefined && emit('update', next[0])"
    >
      <SliderTrack class="relative h-0.5 w-full rounded-full bg-control">
        <SliderRange class="absolute h-full rounded-full bg-foreground" />
      </SliderTrack>
      <SliderThumb
        class="grid size-2.5 place-items-center rounded-full bg-foreground outline-none focus-visible:ring-2 focus-visible:ring-focus-ring"
      >
        <span class="size-0.5 rounded-full bg-raised" />
      </SliderThumb>
    </SliderRoot>
  </label>
</template>
