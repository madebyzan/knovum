<script setup lang="ts">
import { computed, useTemplateRef, watchEffect } from "vue";

import type { VideoAudioClip } from "../project";
import { waveformBars } from "../timeline/waveform";
import { useClipLane } from "./use-clip-lane";
import type { VideoEditor } from "./use-video-editor";

/** CSS pixels per waveform bar, gap included. */
const BAR_PITCH = 3;

const props = defineProps<{ clip: VideoAudioClip; editor: VideoEditor }>();
const { timeline, waveforms, materials, visibleProject } = props.editor;
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const lane = useClipLane(() => props.clip, timeline, canvas);

const asset = computed(() =>
  visibleProject.value?.assets.find(({ nodeId }) => nodeId === props.clip.assetId),
);

watchEffect(() => {
  const media = asset.value;
  if (!media || !materials.available.value.has(props.clip.assetId)) return;
  waveforms.request(media);
  const surface = lane.surface();
  const waveform = waveforms.get(media);
  if (!surface || !waveform || !canvas.value) return;
  const bars = waveformBars(waveform, surface.source, Math.max(1, Math.floor(surface.width / BAR_PITCH)));
  surface.context.fillStyle = globalThis.getComputedStyle(canvas.value).color;
  const middle = surface.height / 2;
  for (const [index, peak] of bars.entries()) {
    const height = Math.max(1, Math.sqrt(peak) * surface.height * 0.9);
    surface.context.fillRect(index * BAR_PITCH, middle - height / 2, BAR_PITCH - 1, height);
  }
});
</script>

<template>
  <canvas
    ref="canvas"
    class="pointer-events-none absolute inset-y-0 text-fg-success/50"
    :style="lane.style.value"
    aria-hidden="true"
  />
</template>
