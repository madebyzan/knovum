<script setup lang="ts">
import { clamp } from "es-toolkit";
import { ProgressIndicator, ProgressRoot } from "reka-ui";
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";

const props = withDefaults(
  defineProps<{
    value?: number | undefined;
    max?: number | undefined;
    label?: string | undefined;
    class?: HTMLAttributes["class"];
  }>(),
  {
    value: 0,
    max: 100,
    label: undefined,
  },
);

const clamped = computed(() => clamp(props.value, 0, props.max));
const fill = computed(() => (clamped.value / props.max) * 100);
</script>

<template>
  <ProgressRoot
    :model-value="clamped"
    :max="max"
    :aria-label="label"
    :class="cn('relative h-1.5 w-full overflow-hidden rounded-full bg-surface-control', props.class)"
  >
    <ProgressIndicator
      class="h-full rounded-full bg-surface-accent transition-[width] duration-150 ease-standard"
      :style="{ width: `${fill}%` }"
    />
  </ProgressRoot>
</template>
