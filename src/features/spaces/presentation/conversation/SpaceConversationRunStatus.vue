<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";

defineProps<{ state?: string | undefined }>();
const emit = defineEmits<{ resume: [] }>();
const { t } = useI18n();
</script>

<template>
  <div
    v-if="state === 'paused'"
    class="flex shrink-0 items-center gap-2 px-3 py-2 text-caption text-fg-warning"
  >
    <span class="min-w-0 flex-1">{{ t("spaces.assistant.tasks.paused") }}</span>
    <UiButton size="sm" variant="secondary" @click="emit('resume')">{{
      t("spaces.assistant.tasks.resume")
    }}</UiButton>
  </div>
  <p v-else-if="state === 'waiting'" class="shrink-0 px-3 py-2 text-caption text-fg-secondary">
    {{ t("spaces.assistant.tasks.waitingForResult") }}
  </p>
</template>
