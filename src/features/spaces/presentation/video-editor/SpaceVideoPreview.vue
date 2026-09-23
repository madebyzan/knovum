<script setup lang="ts">
import { computed } from "vue";

import type { VideoEditProject } from "../../video-editing/project";
import { versionedMediaSource } from "../media/element";

const props = defineProps<{
  sourcePath: string;
  project: VideoEditProject;
  timelineMs: number;
}>();
const video = defineModel<HTMLVideoElement | null>("element", { default: null });
defineEmits<{ playing: [value: boolean]; ready: []; failed: [] }>();
const source = computed(() => versionedMediaSource(props.sourcePath));
const overlay = computed(() =>
  props.project.textTrack.items.find(
    ({ startMs, endMs }) => props.timelineMs >= startMs && props.timelineMs < endMs,
  ),
);
const transform = computed(() => {
  const value = props.project.transform;
  return `rotate(${value.rotation}deg) scale(${value.flipHorizontal ? -1 : 1}, ${value.flipVertical ? -1 : 1})`;
});
</script>

<template>
  <div class="relative grid min-h-0 flex-1 place-items-center overflow-hidden bg-control/25">
    <video
      :ref="(value) => (video = value as HTMLVideoElement | null)"
      :src="source"
      class="max-h-full max-w-full object-contain"
      :style="{ transform }"
      preload="metadata"
      @loadedmetadata="$emit('ready')"
      @error="$emit('failed')"
      @play="$emit('playing', true)"
      @pause="$emit('playing', false)"
    />
    <div
      v-if="overlay"
      class="pointer-events-none absolute inset-x-8 flex justify-center"
      :class="
        overlay.placement === 'top'
          ? 'top-8'
          : overlay.placement === 'center'
            ? 'top-1/2 -translate-y-1/2'
            : 'bottom-8'
      "
    >
      <span
        class="max-w-3xl rounded-control bg-scrim/75 px-3 py-1.5 text-center text-title font-semibold text-white shadow-control"
      >
        {{ overlay.text }}
      </span>
    </div>
  </div>
</template>
