<script setup lang="ts">
import { computed, useTemplateRef, watchEffect } from "vue";
import { useI18n } from "vue-i18n";

import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoClip } from "../project";
import { filmstripStepMs } from "../timeline/filmstrip";
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

/**
 * Draws one thumbnail per slot of the visible slice. A slot is as wide as the source's own frame at
 * the lane's height and shows the frame its slot starts at; slots are anchored to the source's
 * start, so trimming or moving a clip never slides its pictures.
 */
watchEffect(() => {
  const media = asset.value;
  if (!media || media.kind !== "video" || !node.value) return;
  const surface = lane.surface();
  if (!surface) return;
  const slotWidth = Math.max(1, (surface.height * media.width) / Math.max(1, media.height));
  const pxPerMs = timeline.pxPerSecond.value / 1000;
  const slotMs = slotWidth / pxPerMs;
  const stepMs = filmstripStepMs(slotMs);
  const { fromMs, toMs } = surface.source;
  const slots = slotStarts(fromMs, toMs, slotMs);
  const indexOf = (startMs: number) => Math.floor(startMs / stepMs);
  filmstrips.request(media, stepMs, slots.map(indexOf));
  for (const startMs of slots) {
    const tile = filmstrips.tile(media, stepMs, indexOf(startMs));
    if (tile) surface.context.drawImage(tile, (startMs - fromMs) * pxPerMs, 0, slotWidth, surface.height);
  }
});

/** Source times of the slots that intersect `[fromMs, toMs)`, every slot `slotMs` from zero. */
function slotStarts(fromMs: number, toMs: number, slotMs: number) {
  const first = Math.floor(fromMs / slotMs);
  const last = Math.ceil(toMs / slotMs);
  return Array.from({ length: Math.max(0, last - first) }, (_, offset) => (first + offset) * slotMs);
}
</script>

<template>
  <img
    v-if="still"
    :src="still"
    alt=""
    draggable="false"
    class="pointer-events-none absolute inset-0 size-full object-cover"
  />
  <template v-else>
    <canvas
      ref="canvas"
      class="pointer-events-none absolute inset-y-0"
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
