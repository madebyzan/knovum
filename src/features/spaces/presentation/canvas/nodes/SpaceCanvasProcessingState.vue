<script setup lang="ts">
import { CircleAlert } from "@lucide/vue";

import SpaceCanvasWorkEffect from "./SpaceCanvasWorkEffect.vue";

defineProps<{ label: string; phase: "active" | "failed" }>();
</script>

<template>
  <div
    class="work-state-enter relative flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-panel"
    role="status"
    aria-live="polite"
    :aria-label="label"
  >
    <SpaceCanvasWorkEffect v-if="phase === 'active'" placement="field" />
    <span
      v-if="phase === 'active'"
      class="relative inline-flex max-w-[90%] items-center gap-1.5 truncate rounded-full bg-foreground/75 px-3 py-1 text-caption font-medium text-panel"
    >
      <SpaceCanvasWorkEffect placement="inline" />
      <span class="truncate">{{ label }}</span>
    </span>
    <span
      v-else
      class="relative inline-flex max-w-[90%] items-center gap-1.5 rounded-full border border-danger/20 bg-panel px-3 py-1 text-caption font-medium text-danger"
    >
      <CircleAlert class="size-(--knovum-icon-size-sm) shrink-0" :stroke-width="1.6" />
      <span class="truncate">{{ label }}</span>
    </span>
  </div>
</template>

<style scoped>
.work-state-enter {
  animation: work-state-enter 160ms ease-out both;
}

@keyframes work-state-enter {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .work-state-enter {
    animation: none;
  }
}
</style>
