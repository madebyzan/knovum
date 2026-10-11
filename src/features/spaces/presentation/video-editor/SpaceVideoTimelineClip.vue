<script setup lang="ts">
import { clamp } from "es-toolkit";
import { computed } from "vue";

import type { VideoClip } from "../../video-editing/project";
import { formatVideoEditTime } from "../../video-editing/timecode";
import type { TimelineFrame } from "./use-video-timeline-thumbnails";

const props = defineProps<{
  clip: VideoClip;
  selected: boolean;
  frames: TimelineFrame[];
  waveform?: number[] | undefined;
  sourceDurationMs: number;
}>();
const TIMELINE_EDGES = ["start", "end"] as const;
defineEmits<{
  select: [];
  trimStart: [event: PointerEvent, edge: "start" | "end"];
  trimMove: [event: PointerEvent];
  trimEnd: [];
  trimCancel: [];
  nudge: [edge: "start" | "end", delta: number];
}>();

const length = computed(() => props.clip.endMs - props.clip.startMs);
const visibleFrames = computed(() =>
  props.frames
    .filter((frame) => frame.endMs > props.clip.startMs && frame.startMs < props.clip.endMs)
    .map((frame) => {
      const left = Math.max(frame.startMs, props.clip.startMs);
      const right = Math.min(frame.endMs, props.clip.endMs);
      return {
        image: frame.image,
        style: {
          left: `${((left - props.clip.startMs) / length.value) * 100}%`,
          width: `${((right - left) / length.value) * 100}%`,
        },
      };
    }),
);
const peaks = computed(() => {
  const samples = props.waveform;
  if (!samples?.length || !props.sourceDurationMs) return [];
  const start = Math.floor((props.clip.startMs / props.sourceDurationMs) * samples.length);
  const end = Math.ceil((props.clip.endMs / props.sourceDurationMs) * samples.length);
  const count = clamp(end - start, 1, 80);
  return Array.from({ length: count }, (_, index) => {
    const from = start + Math.floor((index / count) * (end - start));
    const to = Math.max(from + 1, start + Math.ceil(((index + 1) / count) * (end - start)));
    return Math.max(0, ...samples.slice(from, to));
  });
});
</script>

<template>
  <div class="relative h-full min-w-0 shrink-0 border-r border-stroke-divider" data-video-clip>
    <div
      class="relative h-16 overflow-hidden border-y bg-surface-accent-subtle"
      :class="
        selected ? 'border-stroke-accent ring-1 ring-stroke-accent ring-inset' : 'border-stroke-accent/30'
      "
    >
      <img
        v-for="(frame, index) in visibleFrames"
        :key="index"
        :src="frame.image"
        :style="frame.style"
        class="pointer-events-none absolute inset-y-0 h-full object-cover opacity-85"
        alt=""
        draggable="false"
      />
      <button
        type="button"
        class="absolute inset-0 w-full px-3 text-left text-caption text-fg-on-image drop-shadow-md focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :aria-label="`${$t('spaces.media.videoEditor.clip.selected')} ${formatVideoEditTime(clip.startMs)}–${formatVideoEditTime(clip.endMs)}`"
        @click="$emit('select')"
      >
        <span class="rounded-md bg-black/45 px-1 py-0.5 tabular-nums">
          {{ formatVideoEditTime(clip.startMs) }} – {{ formatVideoEditTime(clip.endMs) }}
        </span>
      </button>
      <button
        v-for="edge in TIMELINE_EDGES"
        :key="edge"
        type="button"
        class="absolute inset-y-0 z-10 w-2.5 cursor-ew-resize touch-none bg-surface-accent/70 hover:bg-surface-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :class="edge === 'start' ? 'left-0 rounded-l-sm' : 'right-0 rounded-r-sm'"
        :aria-label="
          $t(edge === 'start' ? 'spaces.media.videoEditor.trim.start' : 'spaces.media.videoEditor.trim.end')
        "
        @pointerdown="$emit('trimStart', $event, edge)"
        @pointermove="$emit('trimMove', $event)"
        @pointerup="$emit('trimEnd')"
        @pointercancel="$emit('trimCancel')"
        @keydown.left.prevent="$emit('nudge', edge, -100)"
        @keydown.right.prevent="$emit('nudge', edge, 100)"
      />
    </div>
    <div
      class="flex h-8 items-center gap-px overflow-hidden border-b border-stroke-accent/25 bg-surface-accent-subtle px-0.5"
    >
      <span
        v-for="(peak, index) in peaks"
        :key="index"
        class="min-w-0 flex-1 rounded-full bg-surface-accent/50"
        :style="{ height: `${Math.max(8, Math.sqrt(peak) * 90)}%` }"
      />
    </div>
  </div>
</template>
