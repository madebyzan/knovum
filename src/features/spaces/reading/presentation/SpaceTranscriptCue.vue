<script setup lang="ts">
import SpaceTextQuote from "../../knowledge/presentation/SpaceTextQuote.vue";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceTranscriptSegment } from "../../media/schema/transcript";

defineProps<{
  segment: SpaceTranscriptSegment;
  active: boolean;
  highlighted: boolean;
  quote?: string | undefined;
}>();

const emit = defineEmits<{ seek: [startMs: number] }>();
</script>

<template>
  <div
    :data-segment-start-ms="segment.startMs"
    :data-segment-end-ms="segment.endMs"
    :data-cited-segment="highlighted ? '' : undefined"
    :data-active-segment="active ? '' : undefined"
    class="relative flex items-baseline gap-2.5 rounded-lg px-2.5 py-2 transition-colors duration-120"
    :class="highlighted ? '' : active ? 'bg-surface-control/75' : 'hover:bg-surface-control-hover/55'"
  >
    <button
      data-reader-ignore-selection
      type="button"
      class="w-11 shrink-0 text-left font-mono text-caption-sm tabular-nums transition-colors select-none hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :class="active ? 'font-medium text-fg-accent' : 'text-fg-tertiary'"
      @click="emit('seek', segment.startMs)"
    >
      {{ formatDurationMs(segment.startMs) }}
    </button>
    <div class="min-w-0 flex-1 text-body leading-relaxed text-fg-primary">
      <SpaceTextQuote :text="segment.text" :exact="highlighted ? quote : undefined" />
    </div>
  </div>
</template>
