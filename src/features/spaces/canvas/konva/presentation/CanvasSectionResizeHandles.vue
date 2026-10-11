<script setup lang="ts">
import type Konva from "konva";
import { Circle as KonvaCircle } from "vue-konva/core";

import { canvasChromeMetrics } from "../../canvas-chrome-metrics";
import type { ProjectedSection } from "../../document/structure-scene";
import type { SectionResizeCorner } from "../interaction/structure-gesture-preview";
import type { CanvasPalette } from "./use-canvas-palette";

const corners = ["north-west", "north-east", "south-west", "south-east"] as const;
const props = defineProps<{
  interactive: boolean;
  palette: CanvasPalette;
  section: ProjectedSection;
  zoom: number;
}>();
const emit = defineEmits<{
  resizeStart: [corner: SectionResizeCorner, event: Konva.KonvaEventObject<DragEvent>];
  resizeMove: [event: Konva.KonvaEventObject<DragEvent>];
  gestureEnd: [];
}>();
const width = () => props.section.bounds.right - props.section.bounds.left;
const height = () => props.section.bounds.bottom - props.section.bounds.top;
const metrics = () => canvasChromeMetrics(props.zoom).resize;
</script>

<template>
  <KonvaCircle
    v-for="corner in corners"
    :key="corner"
    :config="{
      x: corner.endsWith('west') ? 0 : width(),
      y: corner.startsWith('north') ? 0 : height(),
      radius: metrics().cornerRadius,
      fill: palette.panel,
      stroke: palette.accent,
      strokeWidth: metrics().strokeWidth,
      hitStrokeWidth: metrics().hitStrokeWidth,
      draggable: interactive,
    }"
    @dragstart="
      $event.cancelBubble = true;
      emit('resizeStart', corner, $event);
    "
    @dragmove="
      $event.cancelBubble = true;
      emit('resizeMove', $event);
    "
    @dragend="
      $event.cancelBubble = true;
      emit('gestureEnd');
    "
  />
</template>
