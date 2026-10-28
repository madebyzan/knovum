<script setup lang="ts">
import { computed, useTemplateRef, watchEffect } from "vue";
import { useI18n } from "vue-i18n";

import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoClip } from "../project";
import { filmstripBucketMs } from "../timeline/filmstrip";
import { useClipLane } from "./use-clip-lane";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ clip: VideoClip; editor: VideoEditor }>();
const { t } = useI18n();
const { timeline, filmstrips, materials, visibleProject, player } = props.editor;
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const lane = useClipLane(() => props.clip, timeline, canvas);

const asset = computed(() =>
  visibleProject.value?.assets.find(({ nodeId }) => nodeId === props.clip.assetId),
);
const node = computed(() => materials.available.value.get(props.clip.assetId));
const still = computed(() =>
  asset.value?.kind === "image" && node.value
    ? versionedMediaSource(node.value.data.assetPath, node.value.data.sourceRevision)
    : undefined,
);
const converting = computed(() => (asset.value ? player.videos.converting(asset.value) : false));

/** Draws the tiles covering the visible slice; only those are requested, never the whole source. */
watchEffect(() => {
  const media = asset.value;
  if (!media || media.kind !== "video" || !node.value) return;
  const surface = lane.surface();
  if (!surface) return;
  const bucketMs = filmstripBucketMs(timeline.pxPerSecond.value);
  const { fromMs, toMs } = surface.source;
  const first = Math.floor(fromMs / bucketMs);
  const indices = Array.from(
    { length: Math.floor(Math.max(fromMs, toMs - 1) / bucketMs) - first + 1 },
    (_, offset) => first + offset,
  );
  filmstrips.request(media, bucketMs, indices);
  const pxPerMs = surface.width / Math.max(1, toMs - fromMs);
  surface.context.imageSmoothingQuality = "high";
  for (const index of indices) {
    const tile = filmstrips.tile(media, bucketMs, index);
    if (!tile) continue;
    drawTile(surface.context, tile, {
      x: (index * bucketMs - fromMs) * pxPerMs,
      width: Math.max(1, bucketMs * pxPerMs),
      height: surface.height,
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
  for (let x = target.x; x < target.x + target.width; x += frameWidth) {
    context.drawImage(tile, 0, 0, tw, th, x, 0, frameWidth, target.height);
  }
  context.restore();
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
  <template v-else>
    <canvas
      ref="canvas"
      class="pointer-events-none absolute inset-y-0 opacity-85"
      :style="lane.style.value"
      aria-hidden="true"
    />
    <span
      v-if="converting"
      role="status"
      class="pointer-events-none absolute bottom-0.5 left-1.5 max-w-full truncate text-caption-sm text-fg-secondary"
    >
      {{ t("spaces.media.videoEditor.preview.converting") }}
    </span>
  </template>
</template>
