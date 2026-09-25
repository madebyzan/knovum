<script setup lang="ts">
import { ChevronDown, ChevronLeft, ChevronRight, ChevronUp } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { ImageEdit } from "../../image-editing/image-edit";
import { clampImageAngle } from "./image-angle-range";
import SpaceImageAngleCameraGuide from "./SpaceImageAngleCameraGuide.vue";
import SpaceImageAngleSubjectGuide from "./SpaceImageAngleSubjectGuide.vue";

const props = defineProps<{
  path: string;
  revision?: string | undefined;
  edit: ImageEdit;
  mode: "subject" | "camera";
  yaw: number;
  pitch: number;
  zoom: "close" | "medium" | "wide";
}>();
const emit = defineEmits<{ update: [yaw: number, pitch: number]; zoom: [delta: number] }>();
const { t } = useI18n();
const gesture = shallowRef<{ pointerId: number; x: number; y: number; yaw: number; pitch: number }>();
let lastWheelAt = 0;

function nudge(yaw: number, pitch: number) {
  emit("update", clampImageAngle(props.yaw + yaw, "yaw"), clampImageAngle(props.pitch + pitch, "pitch"));
}

function begin(event: PointerEvent) {
  if (gesture.value || event.button !== 0) return;
  event.preventDefault();
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  gesture.value = {
    pointerId: event.pointerId,
    x: event.clientX,
    y: event.clientY,
    yaw: props.yaw,
    pitch: props.pitch,
  };
}

function move(event: PointerEvent) {
  if (!gesture.value || gesture.value.pointerId !== event.pointerId) return;
  const sensitivity = event.shiftKey ? 0.15 : 0.5;
  emit(
    "update",
    clampImageAngle(gesture.value.yaw + (event.clientX - gesture.value.x) * sensitivity, "yaw"),
    clampImageAngle(gesture.value.pitch - (event.clientY - gesture.value.y) * sensitivity, "pitch"),
  );
}

function end(event: PointerEvent) {
  if (gesture.value?.pointerId === event.pointerId) gesture.value = undefined;
}

function wheel(event: WheelEvent) {
  event.preventDefault();
  if (Date.now() - lastWheelAt < 180 || Math.abs(event.deltaY) < 8) return;
  lastWheelAt = Date.now();
  emit("zoom", Math.sign(event.deltaY));
}
</script>

<template>
  <div
    class="relative flex aspect-square w-full cursor-grab touch-none items-center justify-center overflow-hidden rounded-panel bg-field active:cursor-grabbing"
    role="group"
    :aria-label="t('spaces.media.anglePreview')"
    @pointerdown="begin"
    @pointermove="move"
    @pointerup="end"
    @pointercancel="end"
    @wheel="wheel"
  >
    <SpaceImageAngleSubjectGuide
      v-if="mode === 'subject'"
      :path="path"
      :revision="revision"
      :edit="edit"
      :yaw="yaw"
      :pitch="pitch"
      :zoom="zoom"
    />
    <SpaceImageAngleCameraGuide
      v-else
      :path="path"
      :revision="revision"
      :edit="edit"
      :yaw="yaw"
      :pitch="pitch"
      :zoom="zoom"
    />
    <button
      v-if="mode === 'camera'"
      type="button"
      class="absolute top-5 left-1/2 -translate-x-1/2 rounded-control p-1 text-foreground-tertiary outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
      :aria-label="t('spaces.media.tiltUp')"
      @pointerdown.stop
      @click="nudge(0, 10)"
    >
      <ChevronUp class="size-(--knovum-icon-size-sm)" />
    </button>
    <button
      v-if="mode === 'camera'"
      type="button"
      class="absolute top-1/2 right-5 -translate-y-1/2 rounded-control p-1 text-foreground-tertiary outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
      :aria-label="t('spaces.media.turnRight')"
      @pointerdown.stop
      @click="nudge(10, 0)"
    >
      <ChevronRight class="size-(--knovum-icon-size-sm)" />
    </button>
    <button
      v-if="mode === 'camera'"
      type="button"
      class="absolute bottom-5 left-1/2 -translate-x-1/2 rounded-control p-1 text-foreground-tertiary outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
      :aria-label="t('spaces.media.tiltDown')"
      @pointerdown.stop
      @click="nudge(0, -10)"
    >
      <ChevronDown class="size-(--knovum-icon-size-sm)" />
    </button>
    <button
      v-if="mode === 'camera'"
      type="button"
      class="absolute top-1/2 left-5 -translate-y-1/2 rounded-control p-1 text-foreground-tertiary outline-none hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring"
      :aria-label="t('spaces.media.turnLeft')"
      @pointerdown.stop
      @click="nudge(-10, 0)"
    >
      <ChevronLeft class="size-(--knovum-icon-size-sm)" />
    </button>
  </div>
</template>
