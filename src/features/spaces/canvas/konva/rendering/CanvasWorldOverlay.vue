<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { CanvasCamera } from "../viewport/canvas-camera";
import { useCameraStyle, worldTransform } from "../viewport/use-camera-style";

/**
 * Canvas-space content that is not a document node — a draft someone is composing in place. It sits
 * above the nodes, follows the camera frame by frame like them, and is left out of thumbnails. It stays
 * interactive while the camera moves, so typing into it is never interrupted by a pan.
 */
const props = defineProps<{ camera: CanvasCamera; renderingEnabled: boolean }>();
const world = useTemplateRef<HTMLElement>("world");
useCameraStyle(props.camera, world, worldTransform);
</script>

<template>
  <div
    :inert="!renderingEnabled"
    data-thumbnail-exclude
    class="pointer-events-none absolute inset-0 z-[18] overflow-hidden"
  >
    <div ref="world" class="absolute top-0 left-0 origin-top-left will-change-transform">
      <slot />
    </div>
  </div>
</template>
