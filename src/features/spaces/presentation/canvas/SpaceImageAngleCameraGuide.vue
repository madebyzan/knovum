<script setup lang="ts">
import { Camera } from "@lucide/vue";
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
    <svg class="absolute inset-0 size-full text-fg-tertiary" viewBox="0 0 100 100" aria-hidden="true">
      <line
        x1="50"
        y1="50"
        :x2="orbit.x"
        :y2="orbit.y"
        stroke="currentColor"
        stroke-width="0.65"
        stroke-dasharray="2 2"
      />
    </svg>
    <div
      class="absolute top-1/2 left-1/2 size-16 -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-md border border-stroke-default bg-surface-raised shadow-control transition-transform duration-150 ease-standard motion-reduce:transition-none"
      :style="subjectStyle"
    >
      <SpaceImageEditedThumbnail :path="path" :revision="revision" :edit="edit" />
      <span
        class="absolute size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full border border-stroke-on-image bg-(--color-content-purple) shadow-control transition-[top,left] duration-150 ease-standard motion-reduce:transition-none"
        :style="gazeStyle"
      />
    </div>
    <div
      class="absolute size-10 -translate-x-1/2 -translate-y-1/2 transition-[top,left,scale] duration-150 ease-standard [perspective:100px] motion-reduce:transition-none"
      :style="cameraStyle"
    >
      <div
        class="relative size-full [transform:rotateX(-22deg)_rotateY(-28deg)] [transform-style:preserve-3d]"
      >
        <div
          class="absolute inset-0 [transform:translateZ(20px)] overflow-hidden rounded-md border border-stroke-strong bg-surface-raised"
        >
          <SpaceImageEditedThumbnail :path="path" :revision="revision" :edit="edit" />
        </div>
        <div
          class="absolute inset-0 grid [transform:rotateY(90deg)_translateZ(20px)] place-items-center rounded-md border border-stroke-strong bg-surface-raised"
        >
          <Camera class="size-3 text-fg-secondary" :stroke-width="1.4" />
        </div>
        <div
          class="absolute inset-0 [transform:rotateX(90deg)_translateZ(20px)] rounded-md border border-stroke-strong bg-surface-raised"
        />
      </div>
    </div>
  </div>
</template>
