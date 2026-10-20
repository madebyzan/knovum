<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { computed, shallowRef, useTemplateRef, watchEffect } from "vue";

import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoClip } from "../project";
import { filmstripBucketMs } from "../timeline/filmstrip";
import { clipLengthMs } from "../timing";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ clip: VideoClip; editor: VideoEditor }>();
const { timeline, filmstrips, materials, visibleProject } = props.editor;
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const size = shallowRef({ width: 0, height: 0 });

const asset = computed(() =>
  visibleProject.value?.assets.find(({ nodeId }) => nodeId === props.clip.assetId),
);
const node = computed(() => materials.available.value.get(props.clip.assetId));
const still = computed(() =>
  asset.value?.kind === "image" && node.value
    ? versionedMediaSource(node.value.data.assetPath, node.value.data.sourceRevision)
    : undefined,
);

useResizeObserver(canvas, ([entry]) => {
  size.value = {
    width: entry?.contentRect.width ?? 0,
    height: entry?.contentRect.height ?? 0,
  };
});

/**
 * Draws the decoded tiles covering the visible part of the clip. Only tiles on screen are
 * requested, so scrolling a long clip never queues a decode of the whole source.
 */
watchEffect(() => {
  const surface = canvas.value;
  const media = asset.value;
  const material = node.value;
  if (!surface || !media || !material || media.kind !== "video" || size.value.width <= 0) return;
  const bucketMs = filmstripBucketMs(timeline.pxPerSecond.value);
  const indices = visibleTiles(bucketMs);
  filmstrips.request(media, bucketMs, indices);
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const cssWidth = Math.max(1, Math.round(size.value.width));
  const cssHeight = Math.max(1, Math.round(size.value.height));
  surface.width = cssWidth * dpr;
  surface.height = cssHeight * dpr;
  const context = surface.getContext("2d");
  if (!context) return;
  context.scale(dpr, dpr);
  context.imageSmoothingEnabled = true;
  context.imageSmoothingQuality = "high";
  context.clearRect(0, 0, cssWidth, cssHeight);
  const pxPerMs = cssWidth / Math.max(1, clipLengthMs(props.clip));
  for (const index of indices) {
    const tile = filmstrips.tile(media, bucketMs, index);
    if (!tile) continue;
    drawTile(context, tile, {
      x: (index * bucketMs - props.clip.sourceInMs) * pxPerMs,
      width: Math.max(1, bucketMs * pxPerMs),
      height: cssHeight,
    });
  }
});

function drawTile(
  context: CanvasRenderingContext2D,
  tile: CanvasImageSource,
  target: { x: number; width: number; height: number },
) {
  const tw = Number((tile as { width?: number }).width) || target.width;
  const th = Number((tile as { height?: number }).height) || target.height;
  const frameWidth = Math.max(1, Math.round(target.height * (tw / th)));
  context.save();
  context.beginPath();
  context.rect(target.x, 0, target.width, target.height);
  context.clip();
  let x = target.x;
  const endX = target.x + target.width;
  while (x < endX) {
    context.drawImage(tile, 0, 0, tw, th, x, 0, frameWidth, target.height);
    x += frameWidth;
  }
  context.restore();
}

/** Tile indices of the clip's source window that currently intersect the timeline viewport. */
function visibleTiles(bucketMs: number) {
  const window = timeline.visible.value;
  const offsetMs = props.clip.sourceInMs - props.clip.timelineStartMs;
  const fromMs = Math.max(props.clip.sourceInMs, window.fromMs + offsetMs);
  const toMs = Math.min(Math.max(props.clip.sourceInMs, props.clip.sourceOutMs - 1), window.toMs + offsetMs);
  if (toMs < fromMs) return [];
  const first = Math.floor(fromMs / bucketMs);
  const last = Math.floor(toMs / bucketMs);
  return Array.from({ length: last - first + 1 }, (_, offset) => first + offset);
}
</script>

<template>
  <img
    v-if="still"
    :src="still"
    alt=""
    draggable="false"
    class="pointer-events-none absolute inset-0 size-full object-cover opacity-80"
  />
  <canvas
    v-else
    ref="canvas"
    class="pointer-events-none absolute inset-0 size-full opacity-85"
    aria-hidden="true"
  />
</template>
