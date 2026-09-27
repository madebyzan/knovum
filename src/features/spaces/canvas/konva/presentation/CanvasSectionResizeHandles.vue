<script setup lang="ts">
import type Konva from "konva";
import { Circle as KonvaCircle } from "vue-konva/core";

import { SECTION_MINIMUM_SIZE } from "../../../model/structure/section-metrics";
import type { ProjectedSection } from "../../document/structure-scene";
import type { CanvasPalette } from "./use-canvas-palette";

type ResizeCorner = "north-west" | "north-east" | "south-west" | "south-east";
const corners = ["north-west", "north-east", "south-west", "south-east"] as const;
const props = defineProps<{
  interactive: boolean;
  palette: CanvasPalette;
  section: ProjectedSection;
  zoom: number;
}>();
const emit = defineEmits<{ resize: [frame: { x: number; y: number; width: number; height: number }] }>();
let origin: { x: number; y: number } | undefined;
const width = () => props.section.bounds.right - props.section.bounds.left;
const height = () => props.section.bounds.bottom - props.section.bounds.top;

function begin(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  origin = { x: event.target.x(), y: event.target.y() };
}

function finish(corner: ResizeCorner, event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  if (!origin) return;
  const delta = { x: event.target.x() - origin.x, y: event.target.y() - origin.y };
  event.target.position(origin);
  origin = undefined;
  const west = corner.endsWith("west");
  const north = corner.startsWith("north");
  const nextWidth = Math.max(SECTION_MINIMUM_SIZE.width, width() + (west ? -delta.x : delta.x));
  const nextHeight = Math.max(SECTION_MINIMUM_SIZE.height, height() + (north ? -delta.y : delta.y));
  const frame = {
    x: west ? props.section.bounds.right - nextWidth : props.section.bounds.left,
    y: north ? props.section.bounds.bottom - nextHeight : props.section.bounds.top,
    width: nextWidth,
    height: nextHeight,
  };
  if (
    frame.x !== props.section.bounds.left ||
    frame.y !== props.section.bounds.top ||
    frame.width !== width() ||
    frame.height !== height()
  )
    emit("resize", frame);
}
</script>

<template>
  <KonvaCircle
    v-for="corner in corners"
    :key="corner"
    :config="{
      x: corner.endsWith('west') ? 0 : width(),
      y: corner.startsWith('north') ? 0 : height(),
      radius: 5 / zoom,
      fill: palette.panel,
      stroke: palette.accent,
      strokeWidth: 1 / zoom,
      hitStrokeWidth: 10 / zoom,
      draggable: interactive,
    }"
    @dragstart="begin"
    @dragend="finish(corner, $event)"
  />
</template>
