<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { CanvasCamera } from "../viewport/canvas-camera";
import { useCameraStyle } from "../viewport/use-camera-style";
import type { CanvasPalette } from "./use-canvas-palette";

const GRID_SPACING = 25;
const DOT_RADIUS = 1.1;

const props = defineProps<{ camera: CanvasCamera; palette: CanvasPalette }>();
const surface = useTemplateRef<HTMLElement>("surface");
const dots = computed(
  () =>
    `radial-gradient(circle at center, ${props.palette.grid} 0 ${DOT_RADIUS}px, transparent ${DOT_RADIUS + 0.4}px)`,
);

/** Tile offset that keeps a dot centered on every world multiple of the grid spacing. */
function tileOffset(translation: number, spacing: number) {
  const offset = (translation - spacing / 2) % spacing;
  return offset < 0 ? offset + spacing : offset;
}

useCameraStyle(props.camera, surface, ({ x, y, zoom }) => {
  const spacing = GRID_SPACING * zoom;
  return {
    backgroundSize: `${spacing}px ${spacing}px`,
    backgroundPosition: `${tileOffset(x, spacing)}px ${tileOffset(y, spacing)}px`,
  };
});
</script>

<template>
  <!--
    The dot grid is screen-space CSS rather than a Konva layer: as a layer it refilled a viewport-sized
    pattern on every camera frame and cost one more full-size scene and hit canvas to clear and composite.
  -->
  <div
    ref="surface"
    aria-hidden="true"
    class="pointer-events-none absolute inset-0 z-0"
    :style="{ backgroundImage: dots }"
  />
</template>
