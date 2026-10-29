<script setup lang="ts">
import { FileClock, GitFork } from "@lucide/vue";

defineProps<{
  nodeWidth: number;
  /** Sources changed since the node was kept; the inspector lists them. */
  sourceUpdates: number;
  /** Outputs derived from the node; selecting it draws their lineage instead. */
  derivedOutputs: number;
}>();
</script>

<template>
  <!-- Pressing a badge presses the node beneath, whose selection shows what the badge stands for. -->
  <div
    v-if="sourceUpdates || derivedOutputs"
    aria-hidden="true"
    class="node-badges pointer-events-none absolute top-0 right-0 flex gap-1 text-caption tabular-nums"
    :style="{ '--node-width': nodeWidth }"
  >
    <span
      v-if="sourceUpdates"
      data-badge="source-updates"
      class="flex h-5 items-center gap-1 rounded-full border border-stroke-warning bg-surface-warning-subtle px-1.5 text-fg-warning shadow-sm"
    >
      <FileClock class="size-3" :stroke-width="1.7" />
      {{ sourceUpdates }}
    </span>
    <span
      v-if="derivedOutputs"
      data-badge="derived-outputs"
      class="flex h-5 items-center gap-1 rounded-full border border-stroke-strong bg-surface-canvas px-1.5 text-fg-secondary shadow-sm"
    >
      <GitFork class="size-3" :stroke-width="1.7" />
      {{ derivedOutputs }}
    </span>
  </div>
</template>

<style scoped>
.node-badges {
  /* Screen-sized at every zoom and set in from the corner; they fade out once the node is too small
     on screen to carry them. `--canvas-zoom` is published by the camera on the world element. */
  transform-origin: top right;
  transform: scale(calc(1 / var(--canvas-zoom))) translate(-6px, 6px);
  opacity: clamp(0, calc((var(--node-width) * var(--canvas-zoom) - 80) / 16), 1);
}
</style>
