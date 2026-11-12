<script setup lang="ts">
import { UserRound } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { formatDurationMs } from "../../media/format-duration";
import type { SpeakerReference } from "../speaker-references";

const props = defineProps<{
  speakers: readonly SpeakerReference[];
  start: string;
  end: string;
  disabled: boolean;
}>();
const emit = defineEmits<{ select: [speaker: SpeakerReference] }>();
const { t } = useI18n();
function selected(speaker: SpeakerReference) {
  return Number(props.start) * 1000 === speaker.startMs && Number(props.end) * 1000 === speaker.endMs;
}
</script>

<template>
  <div class="space-y-1.5">
    <p class="text-caption font-medium text-fg-secondary">{{ t("spaces.speech.reference.speakers") }}</p>
    <div class="flex flex-wrap gap-1.5">
      <button
        v-for="speaker in speakers"
        :key="speaker.speakerId"
        type="button"
        class="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-caption transition-colors disabled:opacity-40"
        :class="
          selected(speaker)
            ? 'border-stroke-strong bg-surface-selected text-fg-primary'
            : 'border-stroke-divider text-fg-secondary hover:bg-surface-control-hover'
        "
        :aria-pressed="selected(speaker)"
        :disabled="disabled"
        :title="t('spaces.speech.reference.speakerHint')"
        @click="emit('select', speaker)"
      >
        <UserRound class="size-3.5" :stroke-width="1.6" />
        <span class="truncate">{{ speaker.speakerId }}</span>
        <span class="text-fg-tertiary tabular-nums">{{
          formatDurationMs(speaker.endMs - speaker.startMs)
        }}</span>
      </button>
    </div>
  </div>
</template>
