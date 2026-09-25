<script setup lang="ts">
import { SliderRoot, SliderThumb, SliderTrack } from "reka-ui";

withDefaults(
  defineProps<{ label: string; value: number; min?: number; colorTrack?: "warmth" | "tint" | undefined }>(),
  { min: -100 },
);
const emit = defineEmits<{ update: [value: number] }>();
</script>

<template>
  <label class="grid gap-1 text-label text-foreground">
    <span class="flex items-center justify-between">
      <span>{{ label }}</span>
      <span class="text-foreground-secondary tabular-nums">{{ value }}</span>
    </span>
    <SliderRoot
      :model-value="[value]"
      :min="min"
      :max="100"
      :step="1"
      :aria-label="label"
      class="relative flex h-5 w-full touch-none items-center select-none"
      @update:model-value="(next) => next?.[0] !== undefined && emit('update', next[0])"
    >
      <SliderTrack
        class="relative h-0.5 w-full rounded-full bg-control"
        :style="
          colorTrack === 'warmth'
            ? { background: 'linear-gradient(90deg, #43adf1, #334155 50%, #f2c31b)' }
            : colorTrack === 'tint'
              ? { background: 'linear-gradient(90deg, #e543bd, #334155 50%, #3cad68)' }
              : undefined
        "
      >
        <span class="absolute top-1/2 left-1/2 h-2 w-px -translate-y-1/2 bg-border-strong" />
      </SliderTrack>
      <SliderThumb
        class="flex size-3.5 items-center justify-center rounded-full border-2 border-raised bg-foreground shadow-control outline-none focus-visible:ring-2 focus-visible:ring-focus-ring"
        ><span class="size-0.5 rounded-full bg-raised"
      /></SliderThumb>
    </SliderRoot>
  </label>
</template>
