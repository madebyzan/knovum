<script setup lang="ts">
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";

defineProps<{ title: string; description: string }>();
</script>

<template>
  <UiSettingsSection>
    <div class="space-y-2 p-4">
      <h2 class="text-body font-medium text-fg-primary">{{ title }}</h2>
      <p class="max-w-md text-caption text-fg-secondary">{{ description }}</p>
      <slot />
    </div>
  </UiSettingsSection>
</template>
