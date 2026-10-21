<script setup lang="ts">
import { GitFork } from "@lucide/vue";

defineProps<{ count: number; nodeWidth: number }>();
</script>

<template>
  <!-- Pressing it presses the node beneath, whose selection draws the lineage this count stands for. -->
  <div
    aria-hidden="true"
    class="derived-outputs pointer-events-none absolute top-0 right-0 flex h-5 items-center gap-1 rounded-full border border-stroke-strong bg-surface-canvas px-1.5 text-caption text-fg-secondary tabular-nums shadow-sm"
    :style="{ '--node-width': nodeWidth }"
  >
    <GitFork class="size-3" :stroke-width="1.7" />
    {{ count }}
  </div>
</template>

<style scoped>
.derived-outputs {
  /* Screen-sized at every zoom and set in from the corner; it fades out once the node is too small
     on screen to carry it. `--canvas-zoom` is published by the camera on the world element. */
  transform-origin: top right;
  transform: scale(calc(1 / var(--canvas-zoom))) translate(-6px, 6px);
  opacity: clamp(0, calc((var(--node-width) * var(--canvas-zoom) - 80) / 16), 1);
}
</style>
