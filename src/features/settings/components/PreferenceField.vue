<script setup lang="ts">
defineProps<{ label: string; description?: string | undefined }>();
</script>

<template>
  <div
    class="flex min-h-(--knovum-preference-item-height) items-center justify-between gap-6 border-b border-divider px-4 py-3 last:border-b-0"
  >
    <div class="min-w-0 flex-1">
      <div class="text-body-lg font-medium text-foreground">{{ label }}</div>
      <div v-if="description" class="mt-0.5 max-w-xl text-caption leading-normal text-foreground-secondary">
        {{ description }}
      </div>
    </div>
    <div class="flex shrink-0 items-center"><slot /></div>
  </div>
</template>
