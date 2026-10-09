<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { computed, shallowRef, toRef, useTemplateRef, watch } from "vue";

import {
  displayedImageRect,
  renderEditedImage,
  type ImageCanvasSize,
} from "../../../image-editing/canvas-image-renderer";
import { imageEditOf, type ImageEdit } from "../../../image-editing/image-edit";
import { useImageEditSession } from "../../../image-editing/image-edit-session";
import { drawRemovalMask } from "../../../knowledge-derivations/image-removal-preview";
import { useImageRemovalSession } from "../../../knowledge-derivations/image-removal-session";
import { versionedMediaSource } from "../../../media/media-element-source";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { useImageSourceDimensions } from "../image-source-dimensions";
import SpaceCanvasImageComments from "./SpaceCanvasImageComments.vue";
import SpaceCanvasImageCropOverlay from "./SpaceCanvasImageCropOverlay.vue";
import SpaceCanvasImageRemovalCursor from "./SpaceCanvasImageRemovalCursor.vue";
import SpaceCanvasImageWorkOverlay from "./SpaceCanvasImageWorkOverlay.vue";
import { useCanvasImageSource } from "./use-canvas-image-source";
import { useImageRemovalStrokes } from "./use-image-removal-strokes";
const props = defineProps<{
  id: string;
  data: SpaceImagePayload;
  interactive: boolean;
  frameRotation: number;
}>();
const session = useImageEditSession();
const dimensions = useImageSourceDimensions();
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
    draw();
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
  const pixelRatio = window.devicePixelRatio || 1;
  size.value = { width: box.width, height: box.height, pixelRatio };
  if (canvas.value) {
    canvas.value.width = Math.max(1, Math.round(box.width * pixelRatio));
    canvas.value.height = Math.max(1, Math.round(box.height * pixelRatio));
  }
  draw();
});
watch(previewEdit, draw, { deep: true });
function draw() {
  const context = canvas.value?.getContext("2d", { willReadFrequently: true });
  if (!context || loadState.value !== "ready") return;
  renderEditedImage(context, image, size.value, previewEdit.value);
  const rect = imageRect.value;
  if (removing.value && rect) {
    const strokes = removalSession.active?.strokes ?? [];
    drawRemovalMask(
      context,
      rect,
      size.value.pixelRatio,
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
    <SpaceCanvasImageWorkOverlay :id="id" :data="data" :load-state="loadState" :image-rect="imageRect" />
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
