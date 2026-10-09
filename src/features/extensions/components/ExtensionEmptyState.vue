<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <div class="space-y-2 py-10 text-center">
    <h2 class="text-body font-medium text-fg-primary">{{ title }}</h2>
    <p class="mx-auto max-w-md text-caption text-fg-secondary">{{ description }}</p>
    <slot />
  </div>
</template>
