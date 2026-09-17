<script setup lang="ts">
import { computed } from "vue";
import { Layer as KonvaLayer, Rect as KonvaRect } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";

import type { CanvasViewport } from "../../contract/canvas-editor-port";
import type { CanvasPalette } from "./use-canvas-palette";

const GRID_SPACING = 25;
const DOT_SCREEN_RADIUS = 1.1;
const PATTERN_PIXEL_RATIO = 4;

const props = defineProps<{
  palette: CanvasPalette;
  size: { width: number; height: number };
  viewport: CanvasViewport;
}>();

let cachedPattern: { color: string; image: HTMLCanvasElement; zoom: number } | undefined;

function gridPattern() {
  const zoom = props.viewport.zoom;
  const color = props.palette.grid;
  if (cachedPattern?.zoom === zoom && cachedPattern.color === color) return cachedPattern.image;
  const canvas = document.createElement("canvas");
  canvas.width = GRID_SPACING * PATTERN_PIXEL_RATIO;
  canvas.height = GRID_SPACING * PATTERN_PIXEL_RATIO;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas grid pattern context is unavailable");
  context.fillStyle = color;
  context.beginPath();
  context.arc(
    canvas.width / 2,
    canvas.height / 2,
    (DOT_SCREEN_RADIUS / zoom) * PATTERN_PIXEL_RATIO,
    0,
    Math.PI * 2,
  );
  context.fill();
  cachedPattern = { color, image: canvas, zoom };
  return canvas;
}

const bounds = computed(() => {
  const { viewport } = props;
  const left = -viewport.x / viewport.zoom;
  const top = -viewport.y / viewport.zoom;
  const right = left + props.size.width / viewport.zoom;
  const bottom = top + props.size.height / viewport.zoom;
  const startX = Math.floor(left / GRID_SPACING) * GRID_SPACING;
  const startY = Math.floor(top / GRID_SPACING) * GRID_SPACING;
  return {
    x: startX - GRID_SPACING / 2,
    y: startY - GRID_SPACING / 2,
    width: right - startX + GRID_SPACING * 1.5,
    height: bottom - startY + GRID_SPACING * 1.5,
    fillPatternImage: gridPattern(),
    fillPatternRepeat: "repeat",
    fillPatternScaleX: 1 / PATTERN_PIXEL_RATIO,
    fillPatternScaleY: 1 / PATTERN_PIXEL_RATIO,
    listening: false,
    perfectDrawEnabled: false,
  };
});
</script>

<template>
  <KonvaLayer :config="{ listening: false, perfectDrawEnabled: false }">
    <KonvaRect :config="bounds" />
  </KonvaLayer>
</template>
