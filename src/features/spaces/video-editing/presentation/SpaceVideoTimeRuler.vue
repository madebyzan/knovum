<script setup lang="ts">
import type { VideoEditProject } from "../project";
import { formatVideoEditTime, formatVideoTimelineTime } from "../timecode";
import { stepOutputFrame } from "../timing";
defineProps<{
  project: VideoEditProject;
  durationMs: number;
  playheadMs: number;
  ticks: readonly number[];
}>();
defineEmits<{
  seek: [milliseconds: number];
  scrubStart: [event: PointerEvent];
  scrubMove: [event: PointerEvent];
}>();
</script>
<template>
  <div
    role="slider"
    data-video-timeline-seek
    tabindex="0"
    class="relative h-8 cursor-crosshair touch-none border-b border-stroke-divider focus-visible:outline-2 focus-visible:outline-stroke-focus"
    :aria-label="$t('spaces.media.videoEditor.timeline.seek')"
    aria-valuemin="0"
    :aria-valuemax="Math.round(durationMs)"
    :aria-valuenow="Math.round(playheadMs)"
    :aria-valuetext="formatVideoEditTime(playheadMs)"
    @pointerdown="$emit('scrubStart', $event)"
    @pointermove="$emit('scrubMove', $event)"
    @keydown.left.prevent.stop="$emit('seek', stepOutputFrame(project, playheadMs, -1))"
    @keydown.right.prevent.stop="$emit('seek', stepOutputFrame(project, playheadMs, 1))"
    @keydown.home.prevent="$emit('seek', 0)"
    @keydown.end.prevent="$emit('seek', durationMs)"
  >
    <span
      v-for="tick in ticks"
      :key="tick"
      class="pointer-events-none absolute top-2 bottom-0 border-l border-stroke-divider"
      :style="{ left: `${(tick / durationMs) * 100}%` }"
      ><span class="absolute -top-1 left-1 text-caption-sm text-fg-tertiary tabular-nums">{{
        tick % 1000 === 0 ? formatVideoTimelineTime(tick) : formatVideoEditTime(tick)
      }}</span></span
    >
  </div>
</template>
