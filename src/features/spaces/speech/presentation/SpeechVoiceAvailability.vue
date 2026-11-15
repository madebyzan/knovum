<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { RouterLink } from "vue-router";

import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";

const props = defineProps<{ voices: readonly VoiceLibraryRevision[] }>();
const { t } = useI18n();
const hasVoice = computed(() => props.voices.some((voice) => voice.isActive && voice.archivedAt == null));
</script>

<template>
  <div
    v-if="!hasVoice"
    class="flex flex-wrap items-center justify-between gap-2 rounded-md bg-surface-control px-2.5 py-2 text-caption text-fg-secondary"
    role="status"
  >
    <span>{{ t("spaces.speech.library.selection.empty") }}</span>
    <RouterLink
      class="shrink-0 font-medium text-fg-accent hover:underline"
      data-testid="manage-voices"
      :to="{ path: '/settings', query: { pane: 'voices' } }"
      >{{ t("spaces.speech.library.manage") }}</RouterLink
    >
  </div>
</template>
