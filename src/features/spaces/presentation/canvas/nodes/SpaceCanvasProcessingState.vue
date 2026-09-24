<script setup lang="ts">
import { CircleAlert } from "@lucide/vue";

import SpaceCanvasWorkDots from "./SpaceCanvasWorkDots.vue";

withDefaults(
  defineProps<{
    label: string;
    failed?: boolean;
  }>(),
  { failed: false },
);
</script>

<template>
  <div class="relative min-h-0 flex-1 overflow-hidden" role="status" aria-live="polite" :aria-label="label">
    <template v-if="failed">
      <div class="grid size-full place-items-center bg-canvas px-4 py-3">
        <div class="flex max-w-64 items-center gap-2 text-caption font-medium text-danger">
          <CircleAlert class="size-(--knovum-icon-size-sm) shrink-0" :stroke-width="1.6" />
          <span class="truncate">{{ label }}</span>
        </div>
      </div>
    </template>
    <div v-else class="flex size-full flex-col items-center justify-center gap-2 bg-canvas">
      <SpaceCanvasWorkDots placement="field" />
      <span class="w-4/5 truncate text-center text-caption text-foreground-tertiary">
        {{ label }}
      </span>
    </div>
  </div>
</template>
