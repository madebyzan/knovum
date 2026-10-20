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
  <p v-if="!hasVoice" class="text-caption text-fg-secondary" role="status">
    {{ t("spaces.speech.library.selection.empty") }}
    <RouterLink
      class="text-fg-accent underline"
      data-testid="manage-voices"
      :to="{ path: '/settings', query: { pane: 'voices' } }"
      >{{ t("spaces.speech.library.manage") }}</RouterLink
    >
  </p>
</template>
