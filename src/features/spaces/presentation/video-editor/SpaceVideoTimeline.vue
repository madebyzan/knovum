<script setup lang="ts">
import { Captions, Film, Volume2 } from "@lucide/vue";
import { computed } from "vue";

import type { VideoEditProject, VideoTextOverlay } from "../../video-editing/project";
import { formatVideoEditTime, formatVideoTimelineTime } from "../../video-editing/timecode";
import SpaceVideoTimelineClip from "./SpaceVideoTimelineClip.vue";
import SpaceVideoTimelineText from "./SpaceVideoTimelineText.vue";
import { useVideoTimelineInteraction, type VideoTimelineProps } from "./use-video-timeline-interaction";
import { useVideoTimelineThumbnails } from "./use-video-timeline-thumbnails";

const props = defineProps<VideoTimelineProps>();
const emit = defineEmits<{
  seek: [milliseconds: number];
  edit: [project: VideoEditProject];
  selectClip: [id: string];
  selectText: [id: string];
}>();
const {
  visibleProject,
  viewport,
  duration,
  ticks,
  contentStyle,
  clipStyle,
  startScrub,
  moveScrub,
  startTrim,
  moveTrim,
  finishTrim,
  cancelTrim,
  nudgeTrim,
} = useVideoTimelineInteraction(props, {
  seek: (milliseconds) => emit("seek", milliseconds),
  trim: (project) => emit("edit", project),
  selectClip: (id) => emit("selectClip", id),
});
const thumbnailSource = computed(() => props.thumbnailSource);
const sourceDuration = computed(() => props.sourceDurationMs);
const { frames, error: thumbnailError } = useVideoTimelineThumbnails(thumbnailSource, sourceDuration);

function editText(edited: VideoTextOverlay) {
  emit("edit", {
    ...props.project,
    textTrack: {
      items: props.project.textTrack.items.map((item) => (item.id === edited.id ? edited : item)),
    },
  });
}
</script>

<template>
  <section
    class="grid h-48 shrink-0 grid-cols-[6rem_minmax(0,1fr)] border-t border-stroke-default bg-surface-panel"
    :aria-label="$t('spaces.media.player.timeline')"
  >
    <div class="border-r border-stroke-divider text-caption text-fg-secondary">
      <div class="h-8 border-b border-stroke-divider" />
      <div class="flex h-11 items-center gap-2 border-b border-stroke-divider px-3">
        <Captions class="size-3.5" />{{ $t("spaces.media.videoEditor.tracks.text") }}
      </div>
      <div class="flex h-16 items-center gap-2 border-b border-stroke-divider px-3">
        <Film class="size-3.5" />{{ $t("spaces.media.videoEditor.tracks.video") }}
      </div>
      <div class="flex h-8 items-center gap-2 px-3">
        <Volume2 class="size-3.5" />{{ $t("spaces.media.videoEditor.tracks.audio") }}
      </div>
    </div>
    <div
      :ref="(value) => (viewport = value as HTMLElement | null)"
      class="min-w-0 overflow-x-auto overflow-y-hidden"
    >
      <div class="relative h-full min-w-full" :style="contentStyle">
        <div
          role="slider"
          data-video-timeline-seek
          tabindex="0"
          class="relative h-8 cursor-crosshair touch-none border-b border-stroke-divider outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus"
          :aria-label="$t('spaces.media.videoEditor.timeline.seek')"
          aria-valuemin="0"
          :aria-valuemax="Math.round(duration)"
          :aria-valuenow="Math.round(playheadMs)"
          :aria-valuetext="formatVideoEditTime(playheadMs)"
          @pointerdown="startScrub"
          @pointermove="moveScrub"
          @keydown.left.prevent="$emit('seek', Math.max(0, playheadMs - 100))"
          @keydown.right.prevent="$emit('seek', Math.min(duration, playheadMs + 100))"
        >
          <span
            v-for="tick in ticks"
            :key="tick"
            class="pointer-events-none absolute top-2 bottom-0 border-l border-stroke-divider"
            :style="{ left: `${(tick / duration) * 100}%` }"
          >
            <span class="absolute -top-1 left-1 text-caption-sm text-fg-tertiary tabular-nums">
              {{ tick % 1_000 === 0 ? formatVideoTimelineTime(tick) : formatVideoEditTime(tick) }}
            </span>
          </span>
        </div>
        <div data-text-track class="relative h-11 border-b border-stroke-divider bg-surface-canvas/40">
          <SpaceVideoTimelineText
            v-for="item in visibleProject.textTrack.items"
            :key="item.id"
            :item="item"
            :duration-ms="duration"
            :selected="selectedTextId === item.id"
            @select="$emit('selectText', item.id)"
            @edit="editText"
          />
        </div>
        <div data-video-track class="relative flex h-24 border-b border-stroke-divider">
          <SpaceVideoTimelineClip
            v-for="clip in visibleProject.videoTrack.clips"
            :key="clip.id"
            :clip="clip"
            :selected="selectedClipId === clip.id"
            :frames="frames"
            :waveform="waveform"
            :source-duration-ms="sourceDurationMs"
            :style="clipStyle(clip.endMs - clip.startMs)"
            @select="$emit('selectClip', clip.id)"
            @trim-start="(event, edge) => startTrim(event, clip, edge)"
            @trim-move="moveTrim"
            @trim-end="finishTrim"
            @trim-cancel="cancelTrim"
            @nudge="(edge, delta) => nudgeTrim(clip, edge, delta)"
          />
          <span
            v-if="thumbnailError"
            class="pointer-events-none absolute right-2 bottom-9 z-20 rounded-md bg-surface-panel/90 px-2 py-0.5 text-caption-sm text-fg-danger"
          >
            {{ $t("spaces.media.videoEditor.timeline.thumbnailFailed") }}
          </span>
        </div>
        <span
          class="pointer-events-none absolute inset-y-0 z-20 w-px bg-surface-danger"
          :style="{ left: `${(playheadMs / duration) * 100}%` }"
        >
          <span class="absolute top-0 -left-1 size-2 rotate-45 bg-surface-danger" />
        </span>
      </div>
    </div>
  </section>
</template>
