<script setup lang="ts">
import { computed, shallowRef } from "vue";

import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import type { VideoAudioClip } from "../project";
import { fadeGain } from "../timing";
import { useVideoPreviewMedia } from "./use-video-preview-media";
const props = defineProps<{
  clip: VideoAudioClip;
  node: VideoMaterialNode;
  timelineMs: number;
  playing: boolean;
}>();
const emit = defineEmits<{ failed: [error: unknown]; ready: [value: boolean] }>();
const element = shallowRef<HTMLAudioElement | null>(null);
const offset = computed(() => props.timelineMs - props.clip.timelineStartMs);
useVideoPreviewMedia({
  element,
  time: () => (props.clip.startMs + offset.value) / 1000,
  gain: () => props.clip.volume * fadeGain(props.clip, offset.value),
  playing: () => props.playing,
  failed: (error) => emit("failed", error),
});
</script>
<template>
  <audio
    :ref="(value) => (element = value as HTMLAudioElement | null)"
    :src="versionedMediaSource(node.data.assetPath, node.data.sourceRevision)"
    crossorigin="anonymous"
    preload="auto"
    @waiting="emit('ready', false)"
    @canplay="emit('ready', true)"
  />
</template>
