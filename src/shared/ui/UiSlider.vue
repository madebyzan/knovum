<script setup lang="ts">
import { SliderRange, SliderRoot, SliderThumb, SliderTrack } from "reka-ui";
import type { HTMLAttributes } from "vue";

import { cn } from "./cn";

const model = defineModel<number>({ required: true });

const props = withDefaults(
  defineProps<{
    label: string;
    min?: number | undefined;
    max?: number | undefined;
    step?: number | undefined;
    disabled?: boolean | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    min: 0,
    max: 100,
    step: 1,
    disabled: false,
  },
);
</script>

<template>
  <SliderRoot
    :model-value="[model]"
    :min="min"
    :max="max"
    :step="step"
    :disabled="disabled"
    :aria-label="label"
    :class="
      cn('relative flex h-5 w-full touch-none items-center select-none data-disabled:opacity-45', props.class)
    "
    @update:model-value="
      (value) => {
        if (value?.[0] !== undefined) model = value[0];
      }
    "
  >
    <SliderTrack class="relative h-1 flex-1 overflow-hidden rounded-full bg-control">
      <SliderRange class="absolute h-full bg-accent" />
    </SliderTrack>
    <SliderThumb
      class="block size-3.5 rounded-full border border-border-strong bg-raised shadow-control transition-colors duration-100 ease-standard outline-none hover:border-foreground focus-visible:border-focus-ring focus-visible:outline-none"
    />
  </SliderRoot>
</template>
