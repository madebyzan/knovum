<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { formatVideoEditTime, formatVideoTimelineTime } from "../timecode";
import { timeToPx } from "../timeline/scale";
import { TIMELINE_ROWS } from "../timeline/track-geometry";
import { projectDurationMs, stepFrame } from "../timing";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, gestures, engine, visibleProject } = props.editor;
const duration = computed(() => (visibleProject.value ? projectDurationMs(visibleProject.value) : 0));

function step(direction: -1 | 1) {
  if (visibleProject.value) engine.seek(stepFrame(visibleProject.value, engine.positionMs(), direction));
}
</script>

<template>
  <div
    role="slider"
    data-video-timeline-seek
    tabindex="0"
    class="absolute inset-x-0 top-0 cursor-col-resize touch-none border-b border-stroke-divider bg-surface-panel/60 focus-visible:outline-2 focus-visible:outline-stroke-focus"
    :style="{ height: `${TIMELINE_ROWS.ruler}px` }"
    :aria-label="t('spaces.media.videoEditor.timeline.seek')"
    aria-valuemin="0"
    :aria-valuemax="Math.round(duration)"
    :aria-valuenow="Math.round(engine.displayMs.value)"
    :aria-valuetext="formatVideoEditTime(engine.displayMs.value)"
    @pointerdown="gestures.begin($event, { kind: 'scrub' })"
    @pointermove="gestures.move"
    @pointerup="gestures.finish"
    @pointercancel="gestures.cancel"
    @keydown.left.prevent.stop="step(-1)"
    @keydown.right.prevent.stop="step(1)"
    @keydown.home.prevent="engine.seek(0)"
    @keydown.end.prevent="engine.seek(duration)"
  >
    <span
      v-for="tick in timeline.ticks.value"
      :key="tick"
      class="pointer-events-none absolute bottom-0 h-2 border-l border-stroke-strong"
      :style="{ left: `${timeToPx(tick, timeline.pxPerSecond.value)}px` }"
    >
      <span class="absolute -top-4 left-1 text-caption-sm text-fg-tertiary tabular-nums">
        {{ formatVideoTimelineTime(tick) }}
      </span>
    </span>
  </div>
</template>
