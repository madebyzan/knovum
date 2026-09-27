<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, type CSSProperties } from "vue";

import {
  cropFromPointer,
  drawCropFromPointer,
  type CropDragHandle,
  type CropDragOrigin,
  type CropDrawOrigin,
} from "../../../image-editing/crop-geometry";
import type { ImageCrop } from "../../../image-editing/image-edit";

interface ImageRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

const props = defineProps<{ crop: ImageCrop; imageRect: ImageRect; lockedAspect?: number }>();
const emit = defineEmits<{ update: [crop: ImageCrop] }>();
const root = useTemplateRef<HTMLDivElement>("root");
type PointerDrag = { kind: "handle"; origin: CropDragOrigin } | { kind: "draw"; origin: CropDrawOrigin };
const drag = shallowRef<PointerDrag>();

const handles: ReadonlyArray<{ handle: Exclude<CropDragHandle, "move">; className: string }> = [
  { handle: "nw", className: "top-0 left-0 -translate-x-1/2 -translate-y-1/2 cursor-nwse-resize" },
  { handle: "n", className: "top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 cursor-ns-resize" },
  { handle: "ne", className: "top-0 right-0 translate-x-1/2 -translate-y-1/2 cursor-nesw-resize" },
  { handle: "e", className: "top-1/2 right-0 translate-x-1/2 -translate-y-1/2 cursor-ew-resize" },
  { handle: "se", className: "right-0 bottom-0 translate-x-1/2 translate-y-1/2 cursor-nwse-resize" },
  { handle: "s", className: "bottom-0 left-1/2 -translate-x-1/2 translate-y-1/2 cursor-ns-resize" },
  { handle: "sw", className: "bottom-0 left-0 -translate-x-1/2 translate-y-1/2 cursor-nesw-resize" },
  { handle: "w", className: "top-1/2 left-0 -translate-x-1/2 -translate-y-1/2 cursor-ew-resize" },
];

const rootStyle = computed<CSSProperties>(() => ({
  left: `${props.imageRect.left}px`,
  top: `${props.imageRect.top}px`,
  width: `${props.imageRect.width}px`,
  height: `${props.imageRect.height}px`,
}));
const cropStyle = computed<CSSProperties>(() => ({
  left: `${props.crop.x * 100}%`,
  top: `${props.crop.y * 100}%`,
  width: `${props.crop.width * 100}%`,
  height: `${props.crop.height * 100}%`,
}));
const shadeStyles = computed<CSSProperties[]>(() => {
  const right = props.crop.x + props.crop.width;
  const bottom = props.crop.y + props.crop.height;
  return [
    { inset: `0 0 auto 0`, height: `${props.crop.y * 100}%` },
    {
      left: 0,
      top: `${props.crop.y * 100}%`,
      width: `${props.crop.x * 100}%`,
      height: `${props.crop.height * 100}%`,
    },
    {
      right: 0,
      top: `${props.crop.y * 100}%`,
      width: `${(1 - right) * 100}%`,
      height: `${props.crop.height * 100}%`,
    },
    { inset: `${bottom * 100}% 0 0 0` },
  ];
});

function begin(handle: CropDragHandle, event: PointerEvent) {
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  drag.value = {
    kind: "handle",
    origin: {
      handle,
      crop: props.crop,
      clientX: event.clientX,
      clientY: event.clientY,
      viewportWidth: props.imageRect.width,
      viewportHeight: props.imageRect.height,
      lockedAspect: props.lockedAspect,
    },
  };
}

function beginDraw(event: PointerEvent) {
  const bounds = root.value!.getBoundingClientRect();
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  drag.value = {
    kind: "draw",
    origin: {
      x: (event.clientX - bounds.left) / bounds.width,
      y: (event.clientY - bounds.top) / bounds.height,
      viewportWidth: bounds.width,
      viewportHeight: bounds.height,
      lockedAspect: props.lockedAspect,
    },
  };
}

function beginInterior(event: PointerEvent) {
  if (props.crop.width === 1 && props.crop.height === 1) beginDraw(event);
  else begin("move", event);
}

function update(event: PointerEvent) {
  const current = drag.value;
  if (!current) return;
  if (current.kind === "handle") {
    emit("update", cropFromPointer(current.origin, event.clientX, event.clientY));
    return;
  }
  const bounds = root.value!.getBoundingClientRect();
  const crop = drawCropFromPointer(
    current.origin,
    (event.clientX - bounds.left) / bounds.width,
    (event.clientY - bounds.top) / bounds.height,
  );
  if (crop) emit("update", crop);
}

function finish() {
  drag.value = undefined;
}

function complete(event: PointerEvent) {
  update(event);
  finish();
}
</script>

<template>
  <div ref="root" class="pointer-events-none absolute z-20" :style="rootStyle">
    <div
      class="pointer-events-auto absolute inset-0 cursor-crosshair touch-none"
      @pointerdown.stop.prevent="beginDraw"
      @pointermove.stop.prevent="update"
      @pointerup.stop="complete"
      @pointercancel="finish"
    />
    <div
      v-for="(style, index) in shadeStyles"
      :key="index"
      class="absolute bg-surface-scrim"
      :style="style"
    />
    <div class="absolute border border-stroke-on-image shadow-control" :style="cropStyle">
      <span class="pointer-events-none absolute top-0 bottom-0 left-1/3 border-l border-stroke-on-image/60" />
      <span class="pointer-events-none absolute top-0 bottom-0 left-2/3 border-l border-stroke-on-image/60" />
      <span class="pointer-events-none absolute top-1/3 right-0 left-0 border-t border-stroke-on-image/60" />
      <span class="pointer-events-none absolute top-2/3 right-0 left-0 border-t border-stroke-on-image/60" />
      <div
        class="pointer-events-auto absolute inset-0 cursor-move touch-none"
        @pointerdown.stop.prevent="beginInterior"
        @pointermove.stop.prevent="update"
        @pointerup.stop="complete"
        @pointercancel="finish"
      />
      <span
        v-for="handle in handles"
        :key="handle.handle"
        aria-hidden="true"
        class="pointer-events-auto absolute z-10 size-2.5 touch-none rounded-full border border-stroke-strong bg-surface-raised shadow-control"
        :class="handle.className"
        @pointerdown.stop.prevent="begin(handle.handle, $event)"
        @pointermove.stop.prevent="update"
        @pointerup.stop="complete"
        @pointercancel="finish"
      />
    </div>
  </div>
</template>
