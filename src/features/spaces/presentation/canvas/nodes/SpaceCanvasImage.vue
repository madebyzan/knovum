<script setup lang="ts">
import { useResizeObserver } from "@vueuse/core";
import { computed, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";

import {
  canvasPointToImage,
  renderEditedImage,
  type ImageCanvasSize,
} from "../../../image-editing/canvas-image-renderer";
import { imageEditOf, type ImageEdit } from "../../../image-editing/image-edit";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { useSpaceStore } from "../../../session/space-store";
import { versionedMediaSource } from "../../media/element";

const props = defineProps<{ id: string; data: SpaceImagePayload; interactive: boolean }>();
const store = useSpaceStore();
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const size = shallowRef<ImageCanvasSize>({ width: 1, height: 1, pixelRatio: 1 });
const image = new Image();
const draftStroke = shallowRef<number[]>();
const source = computed(() => versionedMediaSource(props.data.previewPath ?? props.data.assetPath));
const edit = computed(() => imageEditOf(props.data));
const erasing = computed(() => props.interactive && store.imageEraserNodeId === props.id);

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

watch(
  source,
  (value) => {
    if (value) image.src = value;
  },
  { immediate: true },
);
watch(edit, draw, { deep: true });
image.addEventListener("load", draw);
onBeforeUnmount(() => image.removeEventListener("load", draw));

function draw() {
  const context = canvas.value?.getContext("2d");
  if (!context || !image.complete || !image.naturalWidth) return;
  const next: ImageEdit = draftStroke.value
    ? {
        ...edit.value,
        eraserStrokes: [...edit.value.eraserStrokes, { diameter: 0.045, points: draftStroke.value }],
      }
    : edit.value;
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

async function commitStroke() {
  const points = draftStroke.value;
  draftStroke.value = undefined;
  if (!points || points.length < 4) return draw();
  const before = props.data;
  const after: SpaceImagePayload = {
    ...before,
    imageEdit: { ...edit.value, eraserStrokes: [...edit.value.eraserStrokes, { diameter: 0.045, points }] },
  };
  await store.updateManyData([{ id: props.id, before, after }]);
}
</script>

<template>
  <canvas
    ref="canvas"
    class="block min-h-0 w-full flex-1"
    :class="erasing ? 'nodrag pointer-events-auto cursor-crosshair' : 'pointer-events-none'"
    :aria-label="data.title"
    @pointerdown.stop="beginStroke"
    @pointermove.stop="extendStroke"
    @pointerup.stop="commitStroke"
    @pointercancel="commitStroke"
  />
</template>
