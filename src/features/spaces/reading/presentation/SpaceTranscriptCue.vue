<script setup lang="ts">
import { computed } from "vue";

import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceTranscriptSegment } from "../../media/schema/transcript";
import type { SpeakerBadge } from "../../media/speaker-display";
import { buildTranscriptWordRuns } from "../../media/transcript-word-runs";

const props = defineProps<{
  segment: SpaceTranscriptSegment;
  speaker?: SpeakerBadge | undefined;
  active: boolean;
  currentMs?: number | undefined;
  highlighted: boolean;
  quote?: string | undefined;
}>();

const emit = defineEmits<{ seek: [startMs: number] }>();

const timedRuns = computed(() =>
  props.active && !props.highlighted ? buildTranscriptWordRuns(props.segment) : null,
);

function isActiveWord(startMs: number | null, endMs: number | null) {
  const ms = props.currentMs;
  if (ms === undefined || startMs === null || endMs === null) return false;
  return ms >= startMs && ms < endMs;
}
</script>

<template>
  <div
    :data-segment-start-ms="segment.startMs"
    :data-segment-end-ms="segment.endMs"
    :data-cited-segment="highlighted ? '' : undefined"
    :data-active-segment="active ? '' : undefined"
    class="group relative flex items-start gap-3 rounded-lg px-3 py-2 transition-colors duration-120"
    :class="
      highlighted
        ? ''
        : active
          ? 'bg-surface-control/75 before:pointer-events-none before:absolute before:inset-y-2 before:left-0 before:w-0.5 before:rounded-full before:bg-surface-accent'
          : 'hover:bg-surface-control-hover/55'
    "
  >
    <button
      data-reader-ignore-selection
      type="button"
      class="min-w-10 shrink-0 pt-[3px] text-left font-mono text-caption-sm leading-5 tabular-nums transition-colors select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :class="active ? 'font-medium text-fg-accent' : 'text-fg-tertiary'"
      @click="emit('seek', segment.startMs)"
    >
      {{ formatDurationMs(segment.startMs) }}
    </button>
    <div class="min-w-0 flex-1 space-y-0.5">
      <div v-if="speaker" data-reader-ignore-selection class="flex items-center gap-1.5 select-none">
        <span class="size-1.5 shrink-0 rounded-full" :class="speaker.dotClass" aria-hidden="true" />
        <span class="truncate text-caption-sm font-medium text-fg-secondary">{{ speaker.label }}</span>
      </div>
      <p v-if="timedRuns" class="text-body leading-relaxed break-words whitespace-pre-wrap text-fg-primary">
        <template v-for="run in timedRuns" :key="run.key">
          <template v-if="run.kind === 'gap'">{{ run.text }}</template>
          <span
            v-else
            class="rounded-sm transition-colors duration-75"
            :class="[
              isActiveWord(run.startMs, run.endMs) && 'bg-surface-accent/16 text-fg-accent',
              run.startMs !== null && 'cursor-pointer hover:bg-surface-control-hover/80',
            ]"
            @click="run.startMs !== null && emit('seek', run.startMs)"
            >{{ run.text }}</span
          >
        </template>
      </p>
      <SpaceTextQuote
        v-else
        class="text-body leading-relaxed break-words"
        :text="segment.text"
        :exact="highlighted ? quote : undefined"
      />
    </div>
  </div>
</template>
