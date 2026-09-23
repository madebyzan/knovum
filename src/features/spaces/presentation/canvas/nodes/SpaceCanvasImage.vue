<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { computed, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";

import {
  canvasPointToImage,
  displayedImageRect,
  renderEditedImage,
  type ImageCanvasSize,
} from "../../../image-editing/canvas-image-renderer";
import { imageEditOf, type ImageEdit } from "../../../image-editing/image-edit";
import { useImageEditSession } from "../../../image-editing/image-edit-session";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { versionedMediaSource } from "../../media/element";
import SpaceCanvasImageComments from "./SpaceCanvasImageComments.vue";
import SpaceCanvasImageCropOverlay from "./SpaceCanvasImageCropOverlay.vue";
import SpaceCanvasImageEraserCursor from "./SpaceCanvasImageEraserCursor.vue";
const props = defineProps<{ id: string; data: SpaceImagePayload; interactive: boolean }>();
const session = useImageEditSession();
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const size = shallowRef<ImageCanvasSize>({ width: 1, height: 1, pixelRatio: 1 });
const image = new Image();
image.crossOrigin = "anonymous";
const naturalSize = shallowRef({ width: 0, height: 0 });
const draftStroke = shallowRef<number[]>();
const brushCursor = shallowRef<{ x: number; y: number }>();
const source = computed(() => versionedMediaSource(props.data.previewPath ?? props.data.assetPath));
const edit = computed(() => session.previewFor(props.id) ?? imageEditOf(props.data));
const erasing = computed(
  () => props.interactive && session.active?.nodeId === props.id && session.active.tool === "eraser",
);
const cropping = computed(
  () => props.interactive && session.active?.nodeId === props.id && session.active.tool === "crop",
);
watch(erasing, (value) => {
  if (!value) brushCursor.value = undefined;
});
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
useResizeObserver(canvas, ([entry]) => {
  const box = entry?.contentRect;
  if (!box) return;
  const pixelRatio = window.devicePixelRatio || 1;
  size.value = { width: box.width, height: box.height, pixelRatio };
  if (canvas.value) {
    canvas.value.width = Math.max(1, Math.round(box.width * pixelRatio));
    canvas.value.height = Math.max(1, Math.round(box.height * pixelRatio));
  }
  draw();
});

image.addEventListener("load", imageLoaded);
watch(
  source,
  (value) => {
    naturalSize.value = { width: 0, height: 0 };
    const context = canvas.value?.getContext("2d");
    if (context && canvas.value) context.clearRect(0, 0, canvas.value.width, canvas.value.height);
    if (value) image.src = value;
  },
  { immediate: true },
);
watch(previewEdit, draw, { deep: true });
function imageLoaded() {
  naturalSize.value = { width: image.naturalWidth, height: image.naturalHeight };
  session.setImageDimensions(props.id, image.naturalWidth, image.naturalHeight);
  draw();
}
onBeforeUnmount(() => image.removeEventListener("load", imageLoaded));

function draw() {
  const context = canvas.value?.getContext("2d", { willReadFrequently: true });
  if (!context || !image.complete || !image.naturalWidth) return;
  const next: ImageEdit = draftStroke.value
    ? {
        ...edit.value,
        eraserStrokes: [
          ...edit.value.eraserStrokes,
          { diameter: session.brushDiameter, points: draftStroke.value },
        ],
      }
    : previewEdit.value;
  renderEditedImage(context, image, size.value, next);
}

function point(event: PointerEvent) {
  const bounds = canvas.value!.getBoundingClientRect();
  return canvasPointToImage(
    { x: event.clientX - bounds.left, y: event.clientY - bounds.top },
    image,
    size.value,
    edit.value,
  );
}

function trackBrush(event: PointerEvent) {
  const target = canvas.value;
  if (!target) return;
  const bounds = target.getBoundingClientRect();
  brushCursor.value = {
    x: ((event.clientX - bounds.left) * size.value.width) / bounds.width,
    y: ((event.clientY - bounds.top) * size.value.height) / bounds.height,
  };
}

function beginStroke(event: PointerEvent) {
  if (!erasing.value) return;
  const position = point(event);
  if (!position) return;
  canvas.value?.setPointerCapture(event.pointerId);
  draftStroke.value = position;
  draw();
}

function extendStroke(event: PointerEvent) {
  if (!draftStroke.value) return;
  const position = point(event);
  if (!position) return;
  draftStroke.value = [...draftStroke.value, ...position];
  draw();
}

function commitStroke() {
  const points = draftStroke.value;
  draftStroke.value = undefined;
  if (!points) return;
  session.addEraserStroke(points.length === 2 ? [...points, ...points] : points);
}
</script>

<template>
  <div class="relative min-h-0 w-full flex-1">
    <canvas
      ref="canvas"
      class="absolute inset-0 size-full"
      :class="erasing ? 'nodrag pointer-events-auto cursor-none' : 'pointer-events-none'"
      :aria-label="data.title"
      @pointerdown.stop="beginStroke"
      @pointermove.stop="
        trackBrush($event);
        extendStroke($event);
      "
      @pointerleave="brushCursor = undefined"
      @pointerup.stop="commitStroke"
      @pointercancel="commitStroke"
    />
    <SpaceCanvasImageEraserCursor
      v-if="erasing"
      :position="brushCursor"
      :size="size"
      :natural-size="naturalSize"
      :edit="edit"
      :diameter="session.brushDiameter"
    />
    <SpaceCanvasImageCropOverlay
      v-if="cropping && imageRect"
      :crop="edit.crop"
      :image-rect="imageRect"
      :locked-aspect="session.cropAspectRatio"
      @update="(crop) => session.update({ crop })"
    />
    <SpaceCanvasImageComments
      v-if="imageRect && !cropping && !erasing"
      :id="id"
      :data="data"
      :image-rect="imageRect"
      :interactive="interactive"
    />
  </div>
</template>
