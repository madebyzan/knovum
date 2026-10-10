<script setup lang="ts">
import type { Component } from "vue";

defineProps<{ name: string; description?: string | null | undefined; icon: Component; muted?: boolean }>();
const emit = defineEmits<{ inspect: [] }>();
</script>

<template>
  <div class="flex min-h-(--knovum-setting-min-height) min-w-0 items-center gap-3 py-3">
    <button
      type="button"
      class="group flex min-w-0 flex-1 items-center gap-3 rounded-md text-left outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-stroke-focus"
      :class="muted ? 'text-fg-tertiary' : 'text-fg-primary'"
      @click="emit('inspect')"
    >
      <span
        class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-surface-control text-fg-secondary"
      >
        <component :is="icon" class="size-5" :stroke-width="1.5" />
      </span>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body-lg font-medium group-hover:underline">{{ name }}</span>
        <span
          v-if="description"
          class="mt-0.5 block truncate text-caption text-fg-secondary"
          :title="description"
          >{{ description }}</span
        >
        <slot name="status" />
      </span>
    </button>
    <div v-if="$slots.default" class="flex shrink-0 items-center gap-3"><slot /></div>
  </div>
</template>
