<script setup lang="ts">
import { computed, shallowRef, toRef, useTemplateRef, watch } from "vue";

import type { ImagePixelRect } from "../../../image-editing/image-edit";
import { drawRemovalMask } from "../../../knowledge-derivations/image-removal-preview";
import { useImageSelectionSessionStore } from "../../../knowledge-derivations/image-selection-session-store";
import { imageCanvasHit } from "./image-canvas-hit";
import { drawImageSelection } from "./image-selection-rendering";
import { useCanvasImageSource } from "./use-canvas-image-source";
import { useImageRemovalStrokes } from "./use-image-removal-strokes";

const props = defineProps<{ id: string; rect: ImagePixelRect; frameRotation: number }>();
const selection = useImageSelectionSessionStore();
const canvas = useTemplateRef<HTMLCanvasElement>("canvas");
const active = computed(() => selection.active?.nodeId === props.id);
const brushEnabled = computed(() => active.value && selection.tool === "brush");
const size = computed(() => ({
  width: props.rect.width,
  height: props.rect.height,
  pixelRatio: window.devicePixelRatio,
}));
const localRect = computed(() => ({ left: 0, top: 0, width: props.rect.width, height: props.rect.height }));
const { image, loadState } = useCanvasImageSource(
  toRef(() => selection.maskSource),
  {
    onReady: draw,
    onReset: draw,
  },
);
const brush = useImageRemovalStrokes({
  canvas,
  size,
  rect: localRect,
  enabled: brushEnabled,
  frameRotation: toRef(props, "frameRotation"),
  session: selection,
  redraw: draw,
});
const start = shallowRef<[number, number]>();
const end = shallowRef<[number, number]>();
const pointerId = shallowRef<number>();
function hit(event: PointerEvent) {
  return imageCanvasHit({
    canvas: canvas.value,
    size: size.value,
    rect: localRect.value,
    rotation: props.frameRotation,
    event,
  })?.point;
}
function begin(event: PointerEvent) {
  if (selection.submitting) return;
  if (brushEnabled.value) {
    brush.begin(event);
    return;
  }
  const point = hit(event);
  if (!point) return;
  event.preventDefault();
  if (selection.tool === "box") {
    start.value = point;
    end.value = point;
    pointerId.value = event.pointerId;
    canvas.value?.setPointerCapture(event.pointerId);
  } else selection.addPoint(...point);
}
function move(event: PointerEvent) {
  if (brushEnabled.value) {
    brush.move(event);
    return;
  }
  if (pointerId.value === event.pointerId) {
    end.value = hit(event);
    draw();
  }
}
function finish(event: PointerEvent) {
  if (brushEnabled.value) {
    brush.commit(event);
    return;
  }
  if (pointerId.value !== event.pointerId) return;
  const a = start.value;
  const b = hit(event) ?? end.value;
  if (a && b && Math.abs(b[0] - a[0]) > 0.002 && Math.abs(b[1] - a[1]) > 0.002)
    selection.setBox({
      x: Math.min(a[0], b[0]),
      y: Math.min(a[1], b[1]),
      width: Math.abs(a[0] - b[0]),
      height: Math.abs(a[1] - b[1]),
    });
  reset();
}
function reset() {
  start.value = undefined;
  end.value = undefined;
  pointerId.value = undefined;
  draw();
}
function draw() {
  const element = canvas.value;
  if (!element) return;
  element.width = Math.max(1, Math.round(props.rect.width * size.value.pixelRatio));
  element.height = Math.max(1, Math.round(props.rect.height * size.value.pixelRatio));
  const context = element.getContext("2d");
  if (!context) return;
  drawImageSelection(
    context,
    loadState.value === "ready" ? image : undefined,
    selection.active?.selection,
    size.value,
  );
  const strokes = selection.active?.selection.strokes ?? [];
  drawRemovalMask(
    context,
    localRect.value,
    size.value.pixelRatio,
    brush.draftStroke.value
      ? [...strokes, { diameter: selection.brushDiameter, points: brush.draftStroke.value }]
      : strokes,
  );
  context.scale(size.value.pixelRatio, size.value.pixelRatio);
  context.strokeStyle = "#2d6fef";
  if (start.value && end.value)
    context.strokeRect(
      start.value[0] * props.rect.width,
      start.value[1] * props.rect.height,
      (end.value[0] - start.value[0]) * props.rect.width,
      (end.value[1] - start.value[1]) * props.rect.height,
    );
}
watch([canvas, loadState, () => selection.active?.selection, () => props.rect], draw, {
  deep: true,
  flush: "post",
});
</script>
<template>
  <canvas
    v-if="active"
    ref="canvas"
    class="nodrag absolute z-10 cursor-crosshair touch-none"
    :style="{
      left: `${rect.left}px`,
      top: `${rect.top}px`,
      width: `${rect.width}px`,
      height: `${rect.height}px`,
    }"
    @pointerdown.stop="begin"
    @pointermove.stop="move"
    @pointerup.stop="finish"
    @pointercancel.stop="
      brush.cancel($event);
      reset();
    "
  />
</template>
