<script setup lang="ts">
import type Konva from "konva";
import { nextTick } from "vue";
import {
  Circle as KonvaCircle,
  Group as KonvaGroup,
  Line as KonvaLine,
  Rect as KonvaRect,
  Text as KonvaText,
} from "vue-konva/core";

import { canvasChromeMetrics } from "../../canvas-chrome-metrics";
import type { ProjectedGroup } from "../../document/structure-scene";
import type { CanvasPalette } from "./use-canvas-palette";

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
  transform: [transform: { anchor: { x: number; y: number }; rotationDelta: number; scaleFactor: number }];
}>();
let pressedPoint: { x: number; y: number } | undefined;
let handleOrigin: { point: { x: number; y: number }; center: { x: number; y: number } } | undefined;
const width = () => props.group.bounds.right - props.group.bounds.left;
const height = () => props.group.bounds.bottom - props.group.bounds.top;
const metrics = () => canvasChromeMetrics(props.zoom).group;

function rename() {
  if (props.interactive && !props.group.locked) emit("rename");
}

function pointerDown(event: Konva.KonvaEventObject<PointerEvent>) {
  pressedPoint = event.target.getStage()!.getRelativePointerPosition()!;
  emit("pointerDown", event);
}

function beginTransform(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  handleOrigin = {
    point: pressedPoint!,
    center: { x: props.group.bounds.left + width() / 2, y: props.group.bounds.top + height() / 2 },
  };
  emit("transformStart", event);
}

function previewTransform(kind: "rotate" | "scale", event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  if (!handleOrigin) return;
  const next = event.target.getStage()!.getRelativePointerPosition()!;
  const { point, center } = handleOrigin;
  const initialVector = { x: point.x - center.x, y: point.y - center.y };
  const nextVector = { x: next.x - center.x, y: next.y - center.y };
  const initialDistance = Math.hypot(initialVector.x, initialVector.y);
  emit("transform", {
    anchor: center,
    rotationDelta:
      kind === "rotate"
        ? ((Math.atan2(nextVector.y, nextVector.x) - Math.atan2(initialVector.y, initialVector.x)) * 180) /
          Math.PI
        : 0,
    scaleFactor:
      kind === "scale" && initialDistance
        ? Math.max(0.01, Math.hypot(nextVector.x, nextVector.y) / initialDistance)
        : 1,
  });
  const target = event.target;
  void nextTick(() =>
    target.position(
      kind === "rotate" ? { x: width() / 2, y: -metrics().rotationOffset } : { x: width(), y: height() },
    ),
  );
}

function finishTransform(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  handleOrigin = undefined;
  emit("gestureEnd");
}
</script>

<template>
  <KonvaGroup
    :config="{
      id: `canvas-group-chrome-${group.group.id}`,
      structureNodeId: group.group.id,
      x: group.bounds.left,
      y: group.bounds.top,
      draggable: interactive && !group.locked,
    }"
    @dragstart="emit('dragStart', $event)"
    @dragmove="emit('dragMove', $event)"
    @dragend="emit('gestureEnd')"
    @pointerdown="pointerDown"
    @dblclick="emit('enter')"
  >
    <KonvaRect
      :config="{
        width: width(),
        height: height(),
        fill: 'rgba(0,0,0,0.001)',
        stroke: palette.accent,
        strokeWidth: 2 / zoom,
        dash: [6 / zoom, 4 / zoom],
        hitStrokeWidth: 12 / zoom,
        cornerRadius: 6,
        perfectDrawEnabled: false,
      }"
    />
    <KonvaGroup
      :config="{ x: 8 / zoom, y: -metrics().labelOffset }"
      @dblclick="
        $event.cancelBubble = true;
        rename();
      "
    >
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
        points: [width() - 18 / zoom, 8 / zoom, width() - 8 / zoom, 8 / zoom, width() - 8 / zoom, 18 / zoom],
        stroke: palette.accent,
        strokeWidth: 1.5 / zoom,
        lineCap: 'round',
        lineJoin: 'round',
        listening: false,
      }"
    />
    <KonvaCircle
      v-if="handlesVisible && !group.locked"
      :config="{
        x: width() / 2,
        y: -metrics().rotationOffset,
        radius: metrics().handleRadius,
        fill: palette.panel,
        stroke: palette.accent,
        strokeWidth: metrics().strokeWidth,
        hitStrokeWidth: metrics().hitStrokeWidth,
        draggable: interactive && !group.locked,
      }"
      @dragstart="beginTransform"
      @dragmove="previewTransform('rotate', $event)"
      @dragend="finishTransform"
    />
    <KonvaCircle
      v-if="handlesVisible && !group.locked"
      :config="{
        x: width(),
        y: height(),
        radius: metrics().handleRadius,
        fill: palette.panel,
        stroke: palette.accent,
        strokeWidth: metrics().strokeWidth,
        hitStrokeWidth: metrics().hitStrokeWidth,
        draggable: interactive && !group.locked,
      }"
      @dragstart="beginTransform"
      @dragmove="previewTransform('scale', $event)"
      @dragend="finishTransform"
    />
  </KonvaGroup>
</template>
