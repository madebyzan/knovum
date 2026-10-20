<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { computed, shallowRef, useTemplateRef, watchEffect } from "vue";

import type { VideoAudioClip } from "../project";
import { waveformBars } from "../timeline/waveform";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ clip: VideoAudioClip; editor: VideoEditor }>();
const { waveforms, materials, visibleProject } = props.editor;
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const size = shallowRef({ width: 0, height: 0 });

const asset = computed(() =>
  visibleProject.value?.assets.find(({ nodeId }) => nodeId === props.clip.assetId),
);

useResizeObserver(canvas, ([entry]) => {
  size.value = { width: entry?.contentRect.width ?? 0, height: entry?.contentRect.height ?? 0 };
});

watchEffect(() => {
  const surface = canvas.value;
  const media = asset.value;
  const node = materials.available.value.get(props.clip.assetId);
  if (!surface || !media || !node || size.value.width <= 0) return;
  waveforms.request(media);
  const envelope = waveforms.get(media);
  surface.width = Math.max(1, Math.round(size.value.width));
  surface.height = Math.max(1, Math.round(size.value.height));
  const context = surface.getContext("2d");
  if (!context) return;
  context.clearRect(0, 0, surface.width, surface.height);
  if (!envelope?.length || !media.durationMs) return;
  const bars = waveformBars(
    envelope,
    {
      fromRatio: props.clip.sourceInMs / media.durationMs,
      toRatio: props.clip.sourceOutMs / media.durationMs,
    },
    Math.max(1, Math.floor(surface.width / 3)),
  );
  context.fillStyle = globalThis.getComputedStyle(surface).color;
  const middle = surface.height / 2;
  for (const [index, peak] of bars.entries()) {
    const height = Math.max(1, Math.sqrt(peak) * surface.height * 0.9);
    context.fillRect(index * 3, middle - height / 2, 2, height);
  }
});
</script>

<template>
  <canvas
    ref="canvas"
    class="pointer-events-none absolute inset-0 size-full text-fg-success/50"
    aria-hidden="true"
  />
</template>
