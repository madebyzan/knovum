<script setup lang="ts">
import type Konva from "konva";
import {
  Circle as KonvaCircle,
  Group as KonvaGroup,
  Line as KonvaLine,
  Rect as KonvaRect,
  Text as KonvaText,
} from "vue-konva/core";

import type { ProjectedGroup } from "../../document/structure-scene";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  group: ProjectedGroup;
  interactive: boolean;
  palette: CanvasPalette;
  zoom: number;
}>();
const emit = defineEmits<{
  enter: [];
  move: [delta: { x: number; y: number }];
  movePreview: [delta: { x: number; y: number } | undefined];
  rename: [];
  select: [event: PointerEvent];
  transform: [transform: { anchor: { x: number; y: number }; rotationDelta: number; scaleFactor: number }];
}>();
let origin: { x: number; y: number } | undefined;
let handleOrigin: { x: number; y: number } | undefined;
const width = () => props.group.bounds.right - props.group.bounds.left;
const height = () => props.group.bounds.bottom - props.group.bounds.top;

function beginMove(event: Konva.KonvaEventObject<DragEvent>) {
  origin = { x: event.target.x(), y: event.target.y() };
}

function finishMove(event: Konva.KonvaEventObject<DragEvent>) {
  if (!origin) return;
  const delta = { x: event.target.x() - origin.x, y: event.target.y() - origin.y };
  event.target.position(origin);
  origin = undefined;
  if (delta.x || delta.y) emit("move", delta);
  emit("movePreview", undefined);
}

function previewMove(event: Konva.KonvaEventObject<DragEvent>) {
  if (!origin) return;
  emit("movePreview", { x: event.target.x() - origin.x, y: event.target.y() - origin.y });
}

function select(event: Konva.KonvaEventObject<PointerEvent>) {
  event.cancelBubble = true;
  emit("select", event.evt);
}

function rename() {
  if (props.interactive && !props.group.locked) emit("rename");
}

function beginTransform(event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  handleOrigin = { x: event.target.x(), y: event.target.y() };
}

function finishTransform(kind: "rotate" | "scale", event: Konva.KonvaEventObject<DragEvent>) {
  event.cancelBubble = true;
  if (!handleOrigin) return;
  const next = { x: event.target.x(), y: event.target.y() };
  event.target.position(handleOrigin);
  const center = { x: width() / 2, y: height() / 2 };
  const initialVector = { x: handleOrigin.x - center.x, y: handleOrigin.y - center.y };
  const nextVector = { x: next.x - center.x, y: next.y - center.y };
  handleOrigin = undefined;
  const initialDistance = Math.hypot(initialVector.x, initialVector.y);
  const nextDistance = Math.hypot(nextVector.x, nextVector.y);
  const rotationDelta =
    kind === "rotate"
      ? ((Math.atan2(nextVector.y, nextVector.x) - Math.atan2(initialVector.y, initialVector.x)) * 180) /
        Math.PI
      : 0;
  const scaleFactor =
    kind === "scale" && initialDistance ? Math.max(0.01, nextDistance / initialDistance) : 1;
  if (rotationDelta || scaleFactor !== 1)
    emit("transform", {
      anchor: { x: props.group.bounds.left + center.x, y: props.group.bounds.top + center.y },
      rotationDelta,
      scaleFactor,
    });
}
</script>

<template>
  <KonvaGroup
    :config="{
      id: `canvas-group-chrome-${group.group.id}`,
      x: group.bounds.left,
      y: group.bounds.top,
      draggable: interactive && !group.locked,
    }"
    @dragstart="beginMove"
    @dragmove="previewMove"
    @dragend="finishMove"
    @pointerdown="select"
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
      :config="{ x: 8 / zoom, y: -26 / zoom }"
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
          fontSize: 11 / zoom,
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
      :config="{
        x: width() / 2,
        y: -30 / zoom,
        radius: 5 / zoom,
        fill: palette.panel,
        stroke: palette.accent,
        strokeWidth: 1 / zoom,
        hitStrokeWidth: 10 / zoom,
        draggable: interactive && !group.locked,
      }"
      @dragstart="beginTransform"
      @dragend="finishTransform('rotate', $event)"
    />
    <KonvaCircle
      :config="{
        x: width(),
        y: height(),
        radius: 5 / zoom,
        fill: palette.panel,
        stroke: palette.accent,
        strokeWidth: 1 / zoom,
        hitStrokeWidth: 10 / zoom,
        draggable: interactive && !group.locked,
      }"
      @dragstart="beginTransform"
      @dragend="finishTransform('scale', $event)"
    />
  </KonvaGroup>
</template>
