<script setup lang="ts">
import type { Component } from "vue";

defineProps<{ label: string; description?: string | undefined; icon?: Component | undefined }>();
</script>

<template>
  <div
    class="flex min-h-(--knovum-setting-min-height) flex-wrap items-center justify-between gap-x-6 gap-y-2 border-b border-stroke-divider px-4 py-3 last:border-b-0"
  >
    <div class="flex min-w-40 flex-1 items-center gap-3">
      <component v-if="icon" :is="icon" class="size-4 shrink-0 text-fg-secondary" aria-hidden="true" />
      <div class="min-w-0">
        <div class="text-body-lg font-medium text-fg-primary">{{ label }}</div>
        <div v-if="description" class="mt-0.5 max-w-xl text-caption leading-normal text-fg-secondary">
          {{ description }}
        </div>
      </div>
    </div>
    <div class="flex shrink-0 items-center"><slot /></div>
  </div>
</template>
