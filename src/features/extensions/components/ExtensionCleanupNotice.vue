<script setup lang="ts">
defineProps<{ errors: string[] }>();
</script>

<template>
  <section
    v-if="errors.length"
    role="status"
    class="space-y-1 text-caption text-fg-warning"
    data-testid="extension-cleanup-notice"
  >
    <p>{{ $t("extensions.changeCommittedCleanupPending") }}</p>
    <p v-for="error in errors" :key="error">{{ error }}</p>
  </section>
</template>
