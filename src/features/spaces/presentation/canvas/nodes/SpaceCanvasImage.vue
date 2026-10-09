<script setup lang="ts">
import { error as logError } from "@tauri-apps/plugin-log";
import { useResizeObserver } from "@vueuse/core";
import { computed, inject, shallowRef, toRef, useTemplateRef, watch } from "vue";

import { canvasPreviewZoomKey } from "../../../canvas/preview/viewport-zoom";
import {
  displayedImageRect,
  paintImageBitmap,
  type ImageCanvasSize,
} from "../../../image-editing/canvas-image-renderer";
import { imageEditOf, type ImageEdit } from "../../../image-editing/image-edit";
import { useImageEditSession } from "../../../image-editing/image-edit-session";
import { drawRemovalMask } from "../../../knowledge-derivations/image-removal-preview";
import { useImageRemovalSession } from "../../../knowledge-derivations/image-removal-session";
import { versionedMediaSource } from "../../../media/media-element-source";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { useImageSourceDimensionsStore } from "../image-source-dimensions-store";
import SpaceCanvasImageComments from "./SpaceCanvasImageComments.vue";
import SpaceCanvasImageCropOverlay from "./SpaceCanvasImageCropOverlay.vue";
import SpaceCanvasImageRemovalCursor from "./SpaceCanvasImageRemovalCursor.vue";
import SpaceCanvasImageWorkOverlay from "./SpaceCanvasImageWorkOverlay.vue";
import { useCanvasImageRaster } from "./use-canvas-image-raster";
import { useCanvasImageSource } from "./use-canvas-image-source";
import { useImageRemovalStrokes } from "./use-image-removal-strokes";
const props = defineProps<{
  id: string;
  data: SpaceImagePayload;
  interactive: boolean;
  frameRotation: number;
}>();
const zoom = inject(canvasPreviewZoomKey)!;
const renderFailed = shallowRef(false);
const session = useImageEditSession();
const dimensions = useImageSourceDimensionsStore();
const removalSession = useImageRemovalSession();
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const size = shallowRef<ImageCanvasSize>({ width: 1, height: 1, pixelRatio: 1 });
const source = computed(() =>
  props.data.sourceRevision
    ? versionedMediaSource(props.data.previewPath ?? props.data.assetPath, props.data.sourceRevision)
    : undefined,
);
const { image, naturalSize, loadState } = useCanvasImageSource(source, {
  onReady: (loaded) => {
    if (props.data.sourceRevision)
      dimensions.set(props.id, props.data.sourceRevision, loaded.naturalWidth, loaded.naturalHeight);
    renderFailed.value = false;
    raster.update();
  },
  onReset: () => {
    dimensions.clear(props.id);
    const context = canvas.value?.getContext("2d");
    if (context && canvas.value) context.clearRect(0, 0, canvas.value.width, canvas.value.height);
  },
});
const persistedEdit = computed(() => imageEditOf(props.data));
const edit = computed(() => session.previewFor(props.id) ?? persistedEdit.value);
const removing = computed(
  () =>
    props.interactive && removalSession.active?.nodeId === props.id && removalSession.active.mode === "mask",
);
const cropping = computed(
  () => props.interactive && session.active?.nodeId === props.id && session.active.tool === "crop",
);
const previewEdit = computed<ImageEdit>(() =>
  cropping.value
    ? {
        ...edit.value,
        crop: { x: 0, y: 0, width: 1, height: 1 },
        rotation: 0,
        flipHorizontal: false,
        flipVertical: false,
      }
    : edit.value,
);
const imageRect = computed(() => displayedImageRect(size.value, naturalSize.value, previewEdit.value));
const removal = useImageRemovalStrokes({
  canvas,
  size,
  rect: imageRect,
  enabled: removing,
  frameRotation: toRef(props, "frameRotation"),
  session: removalSession,
  redraw: draw,
});
const brushCursor = removal.cursor;
useResizeObserver(canvas, ([entry]) => {
  const box = entry?.contentRect;
  if (!box) return;
  const pixelRatio = Math.min(window.devicePixelRatio * zoom.value, 2);
  size.value = { width: box.width, height: box.height, pixelRatio };
  raster.update();
});
const raster = useCanvasImageRaster({
  image,
  size: () => size.value,
  edit: () => previewEdit.value,
  ready: () => loadState.value === "ready",
  active: () => session.active?.nodeId === props.id,
  paint: draw,
  error: (error) => {
    renderFailed.value = true;
    void logError(`Image preview failed: ${String(error)}`);
  },
});
watch(source, () => raster.clear());
watch(
  [previewEdit, zoom],
  () => {
    size.value = { ...size.value, pixelRatio: Math.min(window.devicePixelRatio * zoom.value, 2) };
    raster.update();
  },
  { deep: true },
);
function draw() {
  const context = canvas.value?.getContext("2d");
  if (!context || !raster.bitmap.value) return;
  const bitmap = raster.bitmap.value;
  paintImageBitmap(context, bitmap);
  const rect = imageRect.value;
  if (removing.value && rect) {
    const strokes = removalSession.active?.strokes ?? [];
    drawRemovalMask(
      context,
      rect,
      bitmap.width / size.value.width,
      removal.draftStroke.value
        ? [...strokes, { diameter: removalSession.brushDiameter, points: removal.draftStroke.value }]
        : strokes,
    );
  }
}

watch(() => (removalSession.active?.nodeId === props.id ? removalSession.active.strokes : undefined), draw);
</script>

<template>
  <div class="relative min-h-0 w-full flex-1">
    <canvas
      ref="canvas"
      class="absolute inset-0 size-full"
      :class="removing ? 'nodrag pointer-events-auto cursor-none touch-none' : 'pointer-events-none'"
      :aria-label="data.title"
      @pointerdown.stop="removal.begin"
      @pointermove.stop="removal.move"
      @pointerenter="removal.move"
      @pointerleave="removal.leave"
      @pointerup.stop.prevent="removal.commit"
      @pointercancel.stop="removal.cancel"
    />
    <SpaceCanvasImageWorkOverlay
      :id="id"
      :data="data"
      :load-state="renderFailed ? 'failed' : loadState"
      :image-rect="imageRect"
    />
    <SpaceCanvasImageRemovalCursor
      v-if="removing && imageRect"
      v-bind="brushCursor ? { position: brushCursor } : {}"
      :rect="imageRect"
      :diameter="removalSession.brushDiameter"
    />
    <SpaceCanvasImageCropOverlay
      v-if="cropping && imageRect"
      :crop="edit.crop"
      :image-rect="imageRect"
      :frame-rotation="frameRotation"
      v-bind="session.cropAspectRatio === undefined ? {} : { lockedAspect: session.cropAspectRatio }"
      @update="(crop) => session.update({ crop })"
    />
    <SpaceCanvasImageComments
      v-if="imageRect && !cropping && !removing"
      :id="id"
      :data="data"
      :image-rect="imageRect"
      :interactive="interactive"
      :frame-rotation="frameRotation"
    />
  </div>
</template>
