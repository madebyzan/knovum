<script setup lang="ts">
import { CircleHelp } from "@lucide/vue";

import UiPopover from "./UiPopover.vue";

defineProps<{ label: string; explanation?: string; actionLabel?: string | undefined }>();
const emit = defineEmits<{ action: [] }>();
</script>

<template>
  <UiPopover side="top" align="end" class="w-56 p-3">
    <template #trigger>
      <button
        type="button"
        class="grid size-(--knovum-control-height-sm) shrink-0 place-items-center rounded-md text-fg-tertiary outline-none hover:bg-surface-control-hover hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus"
        :aria-label="label"
      >
        <CircleHelp class="size-3.5" :stroke-width="1.6" />
      </button>
    </template>
    <p class="text-label font-medium text-fg-primary">{{ label }}</p>
    <p v-if="explanation" class="mt-1 text-caption leading-relaxed text-fg-secondary">
      {{ explanation }}
    </p>
    <button
      v-if="actionLabel"
      type="button"
      class="mt-2 rounded-md px-1 text-caption font-medium text-fg-accent outline-none hover:underline focus-visible:ring-2 focus-visible:ring-stroke-focus"
      @click="emit('action')"
    >
      {{ actionLabel }}
    </button>
  </UiPopover>
</template>
