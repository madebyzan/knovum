<script setup lang="ts">
import { Layer as KonvaLayer, Rect as KonvaRect } from "vue-konva/core";

import type { CanvasRect } from "../../contract/canvas-editor-port";
import type { CanvasPalette } from "./use-canvas-palette";

defineProps<{
  bounds: readonly CanvasRect[];
  palette: CanvasPalette;
  zoom: number;
}>();
</script>

<template>
  <KonvaLayer :config="{ id: 'canvas-section-candidate-feedback', listening: false }">
    <KonvaRect
      v-for="(frame, index) in bounds"
      :key="index"
      :config="{
        ...frame,
        stroke: palette.accent,
        strokeWidth: 2 / zoom,
        dash: [7 / zoom, 5 / zoom],
        opacity: 0.72,
        cornerRadius: 5 / zoom,
        perfectDrawEnabled: false,
      }"
    />
  </KonvaLayer>
</template>
