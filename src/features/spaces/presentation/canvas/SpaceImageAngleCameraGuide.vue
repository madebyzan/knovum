<script setup lang="ts">
import { range } from "es-toolkit";
import { computed } from "vue";

import type { ImageEdit } from "../../image-editing/image-edit";
import SpaceImageEditedThumbnail from "./SpaceImageEditedThumbnail.vue";

const props = defineProps<{
  path: string;
  revision?: string | undefined;
  edit: ImageEdit;
  yaw: number;
  pitch: number;
  zoom: "close" | "medium" | "wide";
}>();
const rings = range(-75, 90, 15);
const scale = computed(() => (props.zoom === "close" ? 1.18 : props.zoom === "wide" ? 0.84 : 1));
const orbit = computed(() => {
  const yaw = ((props.yaw - 35) * Math.PI) / 180;
  const pitch = (props.pitch * Math.PI) / 180;
  const depth = Math.cos(yaw) * Math.cos(pitch);
  return {
    x: 50 + 38 * Math.sin(yaw) * Math.cos(pitch),
    y: 50 + 13 * depth - 28 * Math.sin(pitch),
    depth,
  };
});
const cameraStyle = computed(() => ({
  left: `${orbit.value.x}%`,
  top: `${orbit.value.y}%`,
  scale: `${0.85 + orbit.value.depth * 0.2}`,
  zIndex: orbit.value.depth >= 0 ? 2 : 0,
}));
const globeStyle = computed(() => ({
  transform: `rotateX(${12 - props.pitch * 0.45}deg) rotateY(${-12 - props.yaw * 0.45}deg)`,
}));
const subjectStyle = computed(() => ({
  transform: `perspective(420px) rotateY(${-props.yaw * 0.32}deg) rotateX(${props.pitch * 0.3}deg) scale(${scale.value})`,
}));
const gazeStyle = computed(() => ({
  left: `${50 - (props.yaw / 90) * 18}%`,
  top: `${50 + (props.pitch / 60) * 18}%`,
}));
</script>

<template>
  <div class="pointer-events-none relative size-52 [perspective:520px]" aria-hidden="true">
    <div
      class="absolute inset-0 transition-transform duration-150 ease-standard [transform-style:preserve-3d] motion-reduce:transition-none"
      :style="globeStyle"
    >
      <div class="absolute inset-0 rounded-full border border-stroke-strong/50" />
      <div
        v-for="ring in rings"
        :key="`longitude-${ring}`"
        class="absolute inset-0 rounded-full border border-stroke-default/60"
        :style="{ transform: `rotateY(${ring}deg)` }"
      />
      <div
        v-for="ring in rings"
        :key="`latitude-${ring}`"
        class="absolute inset-0 rounded-full border border-stroke-default/60"
        :style="{ transform: `rotateX(${ring}deg)` }"
      />
    </div>
    <div
      class="absolute size-12 -translate-x-1/2 -translate-y-1/2 transition-[left,top,scale] duration-150 ease-standard motion-reduce:transition-none"
      :style="cameraStyle"
    >
      <div
        class="absolute inset-x-0 top-1 h-9 rounded-md border border-neutral-950/70 bg-neutral-800 shadow-control"
      >
        <span class="absolute -top-1 left-4 h-1.5 w-4 rounded-t-sm bg-neutral-800" />
        <span class="absolute top-1.5 -right-0.5 h-6 w-1 rounded-r-sm bg-neutral-900" />
        <span class="absolute top-0.5 right-1.5 size-1 rounded-full bg-neutral-500" />
        <div
          class="absolute inset-y-1 left-1 w-8 overflow-hidden rounded-sm border border-neutral-600 bg-neutral-100"
        >
          <SpaceImageEditedThumbnail :path="path" :revision="revision" :edit="edit" />
        </div>
        <span class="absolute top-3 right-1 size-1 rounded-full border border-neutral-400" />
        <span class="absolute right-1 bottom-2 size-1 rounded-full bg-neutral-500" />
      </div>
    </div>
    <div
      class="absolute top-1/2 left-1/2 z-1 size-18 -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-md border border-stroke-strong bg-surface-raised p-0.5 shadow-control transition-transform duration-150 ease-standard motion-reduce:transition-none"
      :style="subjectStyle"
    >
      <SpaceImageEditedThumbnail :path="path" :revision="revision" :edit="edit" />
      <span
        class="absolute size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full border border-stroke-on-image bg-(--color-content-purple) shadow-control transition-[top,left] duration-150 ease-standard motion-reduce:transition-none"
        :style="gazeStyle"
      />
    </div>
  </div>
</template>
