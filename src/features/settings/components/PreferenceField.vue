<script setup lang="ts">
defineProps<{ label: string; description?: string | undefined }>();
</script>

<template>
  <div
    class="flex min-h-(--knovum-setting-min-height) items-center justify-between gap-6 border-b border-stroke-divider px-4 py-3 last:border-b-0"
  >
    <div class="min-w-0 flex-1">
      <div class="text-body-lg font-medium text-fg-primary">{{ label }}</div>
      <div v-if="description" class="mt-0.5 max-w-xl text-caption leading-normal text-fg-secondary">
        {{ description }}
      </div>
    </div>
    <div class="flex shrink-0 items-center"><slot /></div>
  </div>
</template>
