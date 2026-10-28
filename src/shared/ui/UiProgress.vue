<script setup lang="ts">
import { clamp } from "es-toolkit";
import { ProgressIndicator, ProgressRoot } from "reka-ui";
import { computed, type HTMLAttributes } from "vue";

import { cn } from "./cn";

const props = withDefaults(
  defineProps<{
    /** Work done out of `max`; `null` when the amount of work is not known yet. */
    value?: number | null | undefined;
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

const clamped = computed(() => (props.value === null ? null : clamp(props.value, 0, props.max)));
const fill = computed(() => (clamped.value === null ? undefined : `${(clamped.value / props.max) * 100}%`));
</script>

<template>
  <ProgressRoot
    :model-value="clamped"
    :max="max"
    :aria-label="label"
    :class="cn('relative h-1.5 w-full overflow-hidden rounded-full bg-surface-control', props.class)"
  >
    <ProgressIndicator
      :class="
        cn(
          'h-full rounded-full bg-surface-accent',
          clamped === null ? 'ui-progress-sweep w-2/5' : 'transition-[width] duration-150 ease-standard',
        )
      "
      :style="fill === undefined ? undefined : { width: fill }"
    />
  </ProgressRoot>
</template>

<style scoped>
/* Unknown progress sweeps across the track instead of claiming an amount. */
.ui-progress-sweep {
  animation: ui-progress-sweep 1.4s ease-in-out infinite;
}
@keyframes ui-progress-sweep {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(250%);
  }
}
</style>
