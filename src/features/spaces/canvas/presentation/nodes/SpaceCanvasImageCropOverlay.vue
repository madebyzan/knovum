<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, type CSSProperties } from "vue";

import {
  cropFromPointer,
  drawCropFromPointer,
  type CropDragHandle,
  type CropDragOrigin,
  type CropDrawOrigin,
} from "../../../image-editing/crop-geometry";
import type { ImageCrop, ImagePixelRect } from "../../../image-editing/image-edit";
import { viewportToLocalPoint } from "../../viewport-to-local-point";
const props = defineProps<{
  crop: ImageCrop;
  imageRect: ImagePixelRect;
  lockedAspect?: number;
  frameRotation: number;
}>();
const emit = defineEmits<{ update: [crop: ImageCrop] }>();
const root = useTemplateRef<HTMLDivElement>("root");
type PointerDrag = { kind: "handle"; origin: CropDragOrigin } | { kind: "draw"; origin: CropDrawOrigin };
const drag = shallowRef<PointerDrag>();
const HANDLES: ReadonlyArray<{ handle: Exclude<CropDragHandle, "move">; className: string }> = [
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
      inset: `${props.crop.y * 100}% auto auto 0`,
      width: `${props.crop.x * 100}%`,
      height: `${props.crop.height * 100}%`,
    },
    {
      inset: `${props.crop.y * 100}% 0 auto auto`,
      width: `${(1 - right) * 100}%`,
      height: `${props.crop.height * 100}%`,
    },
    { inset: `${bottom * 100}% 0 0 0` },
  ];
});
function begin(handle: CropDragHandle, event: PointerEvent) {
  const point = localPoint(event);
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  drag.value = {
    kind: "handle",
    origin: {
      handle,
      crop: props.crop,
      clientX: point.x,
      clientY: point.y,
      viewportWidth: root.value!.offsetWidth,
      viewportHeight: root.value!.offsetHeight,
      lockedAspect: props.lockedAspect,
    },
  };
}

function beginDraw(event: PointerEvent) {
  const point = localPoint(event);
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  drag.value = {
    kind: "draw",
    origin: {
      x: point.x / root.value!.offsetWidth,
      y: point.y / root.value!.offsetHeight,
      viewportWidth: root.value!.offsetWidth,
      viewportHeight: root.value!.offsetHeight,
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
  const point = localPoint(event);
  if (current.kind === "handle") {
    emit("update", cropFromPointer(current.origin, point.x, point.y));
    return;
  }
  const crop = drawCropFromPointer(
    current.origin,
    point.x / root.value!.offsetWidth,
    point.y / root.value!.offsetHeight,
  );
  if (crop) emit("update", crop);
}

function localPoint(event: PointerEvent) {
  return viewportToLocalPoint(root.value!, event.clientX, event.clientY, props.frameRotation);
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
        v-for="handle in HANDLES"
        :key="handle.handle"
        aria-hidden="true"
        class="pointer-events-auto absolute z-10 size-5 touch-none rounded-full"
        :class="handle.className"
        @pointerdown.stop.prevent="begin(handle.handle, $event)"
        @pointermove.stop.prevent="update"
        @pointerup.stop="complete"
        @pointercancel="finish"
      >
        <span
          class="pointer-events-none absolute inset-1 rounded-full border border-stroke-strong bg-surface-raised shadow-control"
        />
      </span>
    </div>
  </div>
</template>
