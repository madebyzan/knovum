<script setup lang="ts">
import { computed } from "vue";
import { Group as KonvaGroup, Line as KonvaLine, Text as KonvaText } from "vue-konva/core";

import type { CanvasSnapGuide } from "../interaction/canvas-snapping";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{ guide: CanvasSnapGuide; zoom: number; palette: CanvasPalette }>();
const measurement = computed(() => {
  const [x1, y1, x2, y2] = props.guide.points;
  const horizontal = y1 === y2;
  const text = String(Math.round(Math.hypot(x2 - x1, y2 - y1) * 10) / 10);
  const width = Math.max(16, text.length * 7);
  // Dimension text always sits outside its line. No badge, opaque mask, or placement-mode flip near a threshold.
  return {
    text,
    width,
    x: (x1 + x2) / 2 + (horizontal ? -width / 2 : 6) / props.zoom,
    y: (y1 + y2) / 2 - (horizontal ? 17 : 6) / props.zoom,
  };
});
const ticks = computed(() => {
  const [x1, y1, x2, y2] = props.guide.points;
  const half = 3 / props.zoom;
  return y1 === y2
    ? [
        [x1, y1 - half, x1, y1 + half],
        [x2, y2 - half, x2, y2 + half],
      ]
    : [
        [x1 - half, y1, x1 + half, y1],
        [x2 - half, y2, x2 + half, y2],
      ];
});
</script>

<template>
  <KonvaGroup :config="{ listening: false, perfectDrawEnabled: false }">
    <KonvaLine :config="{ points: [...guide.points], stroke: palette.accent, strokeWidth: 1 / zoom }" />
    <KonvaLine
      v-for="(points, index) in ticks"
      :key="index"
      :config="{ points, stroke: palette.accent, strokeWidth: 1 / zoom }"
    />
    <KonvaText
      :config="{
        x: measurement.x,
        y: measurement.y,
        width: measurement.width / zoom,
        height: 12 / zoom,
        text: measurement.text,
        align: 'center',
        fontSize: 11 / zoom,
        fontFamily: palette.typography.fontFamily,
        fill: palette.accent,
      }"
    />
  </KonvaGroup>
</template>
