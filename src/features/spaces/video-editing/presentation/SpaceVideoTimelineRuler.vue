<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { formatVideoEditTime, formatVideoTimelineTick } from "../timecode";
import { timeToPx, timeToTimelineX } from "../timeline/scale";
import { TIMELINE_ROWS } from "../timeline/track-geometry";
import { projectDurationMs, stepFrame } from "../timing";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, gestures, player, visibleProject } = props.editor;
const rulerEl = useTemplateRef<HTMLElement>("rulerEl");
const durationMs = computed(() => (visibleProject.value ? projectDurationMs(visibleProject.value) : 0));
const SUBDIVISIONS = 4;

const minorOffsets = computed(() => {
  const stepPx = timeToPx(timeline.tickStep.value, timeline.pxPerSecond.value);
  if (stepPx < 56) return [];
  return Array.from({ length: SUBDIVISIONS - 1 }, (_, index) => (stepPx * (index + 1)) / SUBDIVISIONS);
});

let lastAriaUpdate = 0;
onScopeDispose(
  player.subscribe((timeMs, playing) => {
    const el = rulerEl.value;
    if (!el) return;
    const now = performance.now();
    if (playing && now - lastAriaUpdate < 250) return;
    lastAriaUpdate = now;
    el.setAttribute("aria-valuenow", String(Math.round(timeMs)));
    el.setAttribute("aria-valuetext", formatVideoEditTime(timeMs));
  }),
);

function step(direction: -1 | 1) {
  if (visibleProject.value) player.seek(stepFrame(visibleProject.value, player.positionMs(), direction));
}
</script>

<template>
  <div
    ref="rulerEl"
    role="slider"
    data-video-timeline-seek
    tabindex="0"
    class="sticky top-0 z-20 w-full cursor-col-resize touch-none border-b border-stroke-divider bg-surface-panel/95 backdrop-blur-xs select-none focus-visible:outline-2 focus-visible:outline-stroke-focus"
    :style="{ height: `${TIMELINE_ROWS.ruler}px` }"
    :aria-label="t('spaces.media.videoEditor.timeline.seek')"
    aria-valuemin="0"
    :aria-valuemax="Math.round(durationMs)"
    aria-valuenow="0"
    aria-valuetext="00:00.000"
    @pointerdown="gestures.begin($event, { kind: 'scrub' })"
    @pointermove="gestures.move"
    @pointerup="gestures.finish"
    @pointercancel="gestures.cancel"
    @keydown.left.prevent.stop="step(-1)"
    @keydown.right.prevent.stop="step(1)"
    @keydown.home.prevent="player.seek(0)"
    @keydown.end.prevent="player.seek(durationMs)"
  >
    <span
      v-for="tick in timeline.ticks.value"
      :key="tick"
      class="pointer-events-none absolute inset-y-0"
      :style="{ left: `${timeToTimelineX(tick, timeline.pxPerSecond.value)}px` }"
    >
      <span
        class="absolute top-1 left-1 font-mono text-caption-sm leading-none text-fg-tertiary tabular-nums"
      >
        {{ formatVideoTimelineTick(tick, timeline.tickStep.value) }}
      </span>
      <span class="absolute bottom-0 left-0 h-2.5 border-l border-stroke-strong/90" />
      <span
        v-for="(offsetPx, subIndex) in minorOffsets"
        :key="subIndex"
        class="absolute bottom-0 h-1.5 border-l border-stroke-divider"
        :style="{ left: `${offsetPx}px` }"
      />
    </span>
  </div>
</template>
