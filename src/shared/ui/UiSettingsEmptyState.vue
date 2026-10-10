<script setup lang="ts">
import type { Component } from "vue";

defineProps<{ icon: Component; message: string }>();
</script>

<template>
  <div class="px-6 py-12 text-center">
    <component :is="icon" class="mx-auto size-4 text-fg-tertiary" :stroke-width="1.5" aria-hidden="true" />
    <p class="mt-3 text-label text-fg-secondary">{{ message }}</p>
  </div>
</template>
