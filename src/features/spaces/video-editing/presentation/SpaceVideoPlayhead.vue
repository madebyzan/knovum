<script setup lang="ts">
import { computed } from "vue";

import { timeToPx } from "../timeline/scale";

const props = defineProps<{ timeMs: number; pxPerSecond: number }>();
const offset = computed(() => timeToPx(props.timeMs, props.pxPerSecond));
</script>

<template>
  <span
    class="pointer-events-none absolute inset-y-0 z-30 w-px bg-surface-danger will-change-transform"
    :style="{ transform: `translate3d(${offset}px, 0, 0)` }"
    aria-hidden="true"
  >
    <span class="absolute top-0 -left-[3px] size-[7px] rounded-b-sm bg-surface-danger" />
  </span>
</template>
