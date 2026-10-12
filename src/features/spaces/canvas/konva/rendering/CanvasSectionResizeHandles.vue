<script setup lang="ts">
import type Konva from "konva";
import { computed, onScopeDispose, watch } from "vue";
import { Circle as KonvaCircle } from "vue-konva/core";

import { canvasChromeMetrics } from "../../canvas-chrome-metrics";
import type { ProjectedSection } from "../../document/structure-scene";
import type { SectionResizeCorner } from "../interaction/structure-gesture-preview";
import type { CanvasPalette } from "./use-canvas-palette";

const CORNERS = ["north-west", "north-east", "south-west", "south-east"] as const;
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
const width = computed(() => props.section.bounds.right - props.section.bounds.left);
const height = computed(() => props.section.bounds.bottom - props.section.bounds.top);
const metrics = computed(() => canvasChromeMetrics(props.zoom).resize);
let container: HTMLDivElement | undefined;

function resetCursor() {
  if (container) container.style.cursor = "";
}
function enter(corner: SectionResizeCorner, event: Konva.KonvaEventObject<PointerEvent>) {
  container = event.target.getStage()?.container();
  if (container && props.interactive)
    container.style.cursor =
      corner === "north-west" || corner === "south-east" ? "nwse-resize" : "nesw-resize";
}
function leave(event: Konva.KonvaEventObject<PointerEvent>) {
  if (!event.target.isDragging()) resetCursor();
}
function beginResize(corner: SectionResizeCorner, event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  emit("resizeStart", corner, event);
}
function moveResize(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  emit("resizeMove", event);
}
function endResize(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  resetCursor();
  emit("gestureEnd");
}
function handleConfig(corner: SectionResizeCorner) {
  return {
    x: corner.endsWith("west") ? 0 : width.value,
    y: corner.startsWith("north") ? 0 : height.value,
    radius: metrics.value.cornerRadius,
    fill: props.palette.accent,
    stroke: props.palette.panel,
    strokeWidth: metrics.value.strokeWidth,
    hitStrokeWidth: metrics.value.hitStrokeWidth,
    draggable: props.interactive,
    onPointerenter: (event: Konva.KonvaEventObject<PointerEvent>) => enter(corner, event),
    onPointerleave: leave,
    onDragstart: (event: Konva.KonvaEventObject<DragEvent>) => beginResize(corner, event),
    onDragmove: moveResize,
    onDragend: endResize,
  };
}
watch(() => props.interactive, resetCursor);
onScopeDispose(resetCursor);
</script>

<template>
  <KonvaCircle v-for="corner in CORNERS" :key="corner" :config="handleConfig(corner)" />
</template>
