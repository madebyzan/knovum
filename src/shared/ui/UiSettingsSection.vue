<script setup lang="ts">
defineProps<{ title?: string; description?: string }>();
</script>

<template>
  <section class="min-w-0">
    <header v-if="title || description" class="mb-2 px-0.5">
      <h2 class="m-0 text-title-sm font-semibold tracking-title text-fg-primary">{{ title }}</h2>
      <p v-if="description" class="mt-0.5 text-caption text-fg-secondary">{{ description }}</p>
    </header>
    <div class="min-w-0 overflow-hidden rounded-panel border border-stroke-default bg-surface-raised">
      <slot />
    </div>
  </section>
</template>
