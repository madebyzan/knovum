<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import { previewGeometry, type previewFrames } from "../preview-composition";
import type { VideoAsset, VideoEditProject } from "../project";
import { useVideoEditProxy } from "../use-video-edit-proxy";
import { useVideoPreviewMedia } from "./use-video-preview-media";

const props = defineProps<{
  frame: ReturnType<typeof previewFrames>[number];
  asset: VideoAsset;
  node: VideoMaterialNode;
  output: VideoEditProject["output"];
  scale: number;
  spaceId: string;
  playing: boolean;
}>();
const emit = defineEmits<{ failed: [error: unknown]; ready: [value: boolean] }>();
const element = shallowRef<HTMLVideoElement | null>(null);
const proxy = useVideoEditProxy(
  computed(() => props.asset.nodeId),
  computed(() => props.asset.sourceRevision),
  computed(() => props.spaceId),
  computed(() => props.asset.kind === "video"),
);
const source = computed(() =>
  props.asset.kind === "video"
    ? versionedMediaSource(proxy.path.value, props.asset.sourceRevision)
    : versionedMediaSource(props.node.data.assetPath, props.asset.sourceRevision),
);

const geometry = computed(() => {
  const { width, height } = previewGeometry(props.asset, props.frame.clip.transform, props.output);
  const transform = props.frame.clip.transform;
  return {
    width: `${width * props.scale}px`,
    height: `${height * props.scale}px`,
    opacity: props.frame.fade,
    transform: `translate(-50%, -50%) scale(${transform.flipHorizontal ? -1 : 1}, ${transform.flipVertical ? -1 : 1}) rotate(${transform.rotation}deg)`,
  };
});
useVideoPreviewMedia({
  element,
  time: () => (props.frame.clip.startMs + props.frame.offsetMs) / 1000,
  gain: () => props.frame.gain,
  playing: () => props.playing,
  failed: (error) => emit("failed", error),
});
watch(source, () => emit("ready", false));
watch(proxy.error, (error) => {
  if (error) emit("failed", error);
});
</script>
<template>
  <div
    class="absolute inset-0 overflow-hidden bg-black"
    :style="{
      opacity: frame.opacity,
      clipPath: frame.clipPath,
      mixBlendMode: frame.dissolve ? 'plus-lighter' : undefined,
    }"
  >
    <img
      v-if="asset.kind === 'image'"
      :src="source"
      alt=""
      draggable="false"
      class="absolute top-1/2 left-1/2 max-w-none"
      :style="geometry"
      @load="emit('ready', true)"
      @error="emit('failed', new Error('Image preview is unavailable'))"
    />
    <video
      v-else
      :ref="(value) => (element = value as HTMLVideoElement | null)"
      :src="source"
      crossorigin="anonymous"
      preload="auto"
      class="absolute top-1/2 left-1/2 max-w-none"
      :style="geometry"
      @waiting="emit('ready', false)"
      @canplay="emit('ready', true)"
    />
  </div>
</template>
