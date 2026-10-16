<script setup lang="ts">
import { computed } from "vue";
import { Group as KonvaGroup, Line as KonvaLine, Rect as KonvaRect, Text as KonvaText } from "vue-konva/core";

import type { CanvasSnapGuide } from "../interaction/canvas-snapping";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{ guide: CanvasSnapGuide; zoom: number; palette: CanvasPalette }>();
const measurement = computed(() => {
  const [x1, y1, x2, y2] = props.guide.points;
  const horizontal = y1 === y2;
  const distance = Math.hypot(x2 - x1, y2 - y1);
  const text = String(Math.round(distance * 10) / 10);
  const width = Math.max(24, text.length * 7 + 12);
  // A tiny gap must not have its entire line and endpoint ticks obscured by the badge.
  const short = distance * props.zoom < (horizontal ? width + 12 : 30);
  return {
    horizontal,
    text,
    width,
    x: (x1 + x2) / 2 + (!horizontal && short ? (width / 2 + 8) / props.zoom : 0),
    y: (y1 + y2) / 2 - (horizontal && short ? 18 / props.zoom : 0),
  };
});
const ticks = computed(() => {
  const [x1, y1, x2, y2] = props.guide.points;
  const half = 3 / props.zoom;
  return measurement.value.horizontal
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
    <KonvaGroup :config="{ x: measurement.x, y: measurement.y }">
      <KonvaRect
        :config="{
          x: -measurement.width / 2 / zoom,
          y: -9 / zoom,
          width: measurement.width / zoom,
          height: 18 / zoom,
          fill: palette.panel,
          stroke: palette.accent,
          strokeWidth: 1 / zoom,
          cornerRadius: 4 / zoom,
        }"
      />
      <KonvaText
        :config="{
          x: -measurement.width / 2 / zoom,
          y: -6 / zoom,
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
  </KonvaGroup>
</template>
