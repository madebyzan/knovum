<script setup lang="ts">
import { CircleAlert, LoaderCircle } from "@lucide/vue";

withDefaults(
  defineProps<{
    label: string;
    failed?: boolean;
  }>(),
  { failed: false },
);
</script>

<template>
  <div class="grid min-h-0 flex-1 place-items-center bg-canvas/35 px-4 py-3" role="status" aria-live="polite">
    <div
      class="flex max-w-64 items-center gap-2 text-caption font-medium"
      :class="failed ? 'text-danger' : 'text-foreground-secondary'"
    >
      <CircleAlert v-if="failed" class="size-(--knovum-icon-size-sm) shrink-0" :stroke-width="1.6" />
      <LoaderCircle
        v-else
        class="size-(--knovum-icon-size-sm) shrink-0 animate-spin text-accent motion-reduce:animate-none"
        :stroke-width="1.6"
      />
      <span class="truncate">{{ label }}</span>
    </div>
  </div>
</template>
