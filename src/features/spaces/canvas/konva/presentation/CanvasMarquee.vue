<script setup lang="ts">
import { Layer as KonvaLayer, Rect as KonvaRect } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";

import type { CanvasRect } from "../../contract/canvas-editor-port";
import type { CanvasPalette } from "./use-canvas-palette";

defineProps<{ bounds: CanvasRect | undefined; palette: CanvasPalette; zoom: number }>();
</script>

<template>
  <KonvaLayer v-if="bounds" :config="{ listening: false }">
    <KonvaRect
      :config="{
        ...bounds,
        fill: palette.accent,
        opacity: 0.08,
        stroke: palette.accent,
        strokeWidth: 1 / zoom,
      }"
    />
  </KonvaLayer>
</template>
