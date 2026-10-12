<script setup lang="ts">
import type Konva from "konva";
import { computed, nextTick } from "vue";
import {
  Circle as KonvaCircle,
  Group as KonvaGroup,
  Line as KonvaLine,
  Rect as KonvaRect,
  Text as KonvaText,
} from "vue-konva/core";

import { canvasChromeMetrics } from "../../canvas-chrome-metrics";
import type { ProjectedGroup } from "../../document/structure-scene";
import {
  groupHandleTransform,
  type GroupHandleOrigin,
  type GroupHandleTransform,
} from "../interaction/group-handle-transform";
import type { CanvasPalette } from "./use-canvas-palette";

type GroupHandleKind = "rotate" | "scale";
const HANDLES: readonly GroupHandleKind[] = ["rotate", "scale"];
const props = defineProps<{
  group: ProjectedGroup;
  interactive: boolean;
  handlesVisible: boolean;
  palette: CanvasPalette;
  zoom: number;
}>();
const emit = defineEmits<{
  enter: [];
  dragStart: [event: Konva.KonvaEventObject<DragEvent>];
  dragMove: [event: Konva.KonvaEventObject<DragEvent>];
  gestureEnd: [];
  transformStart: [event: Konva.KonvaEventObject<DragEvent>];
  rename: [];
  pointerDown: [event: Konva.KonvaEventObject<PointerEvent>];
  transform: [transform: GroupHandleTransform];
}>();
let pressedPoint: { x: number; y: number } | undefined;
let handleOrigin: GroupHandleOrigin | undefined;
const width = computed(() => props.group.bounds.right - props.group.bounds.left);
const height = computed(() => props.group.bounds.bottom - props.group.bounds.top);
const metrics = computed(() => canvasChromeMetrics(props.zoom).group);

function pointerDown(event: Konva.KonvaEventObject<PointerEvent>) {
  pressedPoint = event.target.getStage()!.getRelativePointerPosition()!;
  emit("pointerDown", event);
}

function beginTransform(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  handleOrigin = {
    point: pressedPoint!,
    center: { x: props.group.bounds.left + width.value / 2, y: props.group.bounds.top + height.value / 2 },
  };
  emit("transformStart", event);
}

function previewTransform(kind: GroupHandleKind, event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  if (!handleOrigin) return;
  const next = event.target.getStage()!.getRelativePointerPosition()!;
  emit("transform", groupHandleTransform(kind, handleOrigin, next));
  const target = event.target;
  void nextTick(() => target.position(handlePosition(kind)));
}

function finishTransform(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  handleOrigin = undefined;
  emit("gestureEnd");
}

function handlePosition(kind: GroupHandleKind) {
  if (kind === "rotate") return { x: width.value / 2, y: -metrics.value.rotationOffset };
  return { x: width.value, y: height.value };
}

const chromeConfig = computed(() => ({
  id: `canvas-group-chrome-${props.group.group.id}`,
  structureNodeId: props.group.group.id,
  x: props.group.bounds.left,
  y: props.group.bounds.top,
  draggable: props.interactive && !props.group.locked,
  onDragstart: (event: Konva.KonvaEventObject<DragEvent>) => emit("dragStart", event),
  onDragmove: (event: Konva.KonvaEventObject<DragEvent>) => emit("dragMove", event),
  onDragend: () => emit("gestureEnd"),
  onPointerdown: pointerDown,
  onDblclick: () => emit("enter"),
}));

const labelConfig = computed(() => ({
  x: 8 / props.zoom,
  y: -metrics.value.labelOffset,
  onDblclick: (event: Konva.KonvaEventObject<MouseEvent>) => {
    event.cancelBubble = true;
    if (props.interactive && !props.group.locked) emit("rename");
  },
}));

function handleConfig(kind: GroupHandleKind) {
  return {
    ...handlePosition(kind),
    radius: metrics.value.handleRadius,
    fill: props.palette.panel,
    stroke: props.palette.accent,
    strokeWidth: metrics.value.strokeWidth,
    hitStrokeWidth: metrics.value.hitStrokeWidth,
    draggable: props.interactive && !props.group.locked,
    onDragstart: beginTransform,
    onDragmove: (event: Konva.KonvaEventObject<DragEvent>) => previewTransform(kind, event),
    onDragend: finishTransform,
  };
}
</script>

<template>
  <KonvaGroup :config="chromeConfig">
    <KonvaRect
      :config="{
        width: width,
        height: height,
        fill: 'rgba(0,0,0,0.001)',
        stroke: palette.accent,
        strokeWidth: 2 / zoom,
        dash: [6 / zoom, 4 / zoom],
        hitStrokeWidth: 12 / zoom,
        cornerRadius: 6,
        perfectDrawEnabled: false,
      }"
    />
    <KonvaGroup :config="labelConfig">
      <KonvaRect
        :config="{
          width: Math.max(72, group.group.title.length * 7 + 20) / zoom,
          height: 22 / zoom,
          fill: palette.accent,
          cornerRadius: 4 / zoom,
        }"
      />
      <KonvaText
        :config="{
          x: 8 / zoom,
          y: 5 / zoom,
          width: Math.max(52, group.group.title.length * 7) / zoom,
          text: group.group.title,
          fill: palette.canvas,
          fontFamily: palette.typography.fontFamily,
          fontSize: palette.typography.captionSize / zoom,
          ellipsis: true,
          wrap: 'none',
        }"
      />
    </KonvaGroup>
    <KonvaLine
      :config="{
        points: [width - 18 / zoom, 8 / zoom, width - 8 / zoom, 8 / zoom, width - 8 / zoom, 18 / zoom],
        stroke: palette.accent,
        strokeWidth: 1.5 / zoom,
        lineCap: 'round',
        lineJoin: 'round',
        listening: false,
      }"
    />
    <template v-if="handlesVisible && !group.locked">
      <KonvaCircle v-for="kind in HANDLES" :key="kind" :config="handleConfig(kind)" />
    </template>
  </KonvaGroup>
</template>
