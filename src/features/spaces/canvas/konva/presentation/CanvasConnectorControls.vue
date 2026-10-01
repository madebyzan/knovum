<script setup lang="ts">
import type Konva from "konva";
import { computed, shallowRef } from "vue";
import { Arrow as KonvaArrow, Circle as KonvaCircle, Group as KonvaGroup } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Arrow";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Circle";

import { canvasChromeMetrics } from "../../canvas-chrome-metrics";
import type { CanvasSceneNode } from "../scene/canvas-scene";
import { connectorControlCenter, type CanvasConnectorSide } from "../scene/connector-geometry";
import type { CanvasPalette } from "./use-canvas-palette";

const SIDES: readonly CanvasConnectorSide[] = ["top", "right", "bottom", "left"];
const props = defineProps<{
  interactive: boolean;
  node?: CanvasSceneNode | undefined;
  palette: CanvasPalette;
  visible: boolean;
  zoom: number;
}>();
const emit = defineEmits<{ start: [nodeId: string, side: CanvasConnectorSide] }>();
const hoveredSide = shallowRef<CanvasConnectorSide>();
const metrics = computed(() => canvasChromeMetrics(props.zoom).connector);
const controls = computed(() => {
  const node = props.node;
  if (!node) return [];
  return SIDES.map((side) => ({
    nodeId: node.node.id,
    side,
    position: connectorControlCenter(node.layout.frame, side, metrics.value.offset),
  }));
});

function begin(control: (typeof controls.value)[number], event: Konva.KonvaEventObject<PointerEvent>) {
  event.cancelBubble = true;
  hoveredSide.value = undefined;
  emit("start", control.nodeId, control.side);
}

function setHovered(side: CanvasConnectorSide | undefined, event: Konva.KonvaEventObject<PointerEvent>) {
  hoveredSide.value = side;
  const container = event.target.getStage()?.container();
  if (container) container.style.cursor = side ? "pointer" : "";
}

function points(side: CanvasConnectorSide) {
  const length = metrics.value.arrowHalfLength;
  if (side === "top") return [0, length, 0, -length];
  if (side === "right") return [-length, 0, length, 0];
  if (side === "bottom") return [0, -length, 0, length];
  return [length, 0, -length, 0];
}
</script>

<template>
  <KonvaGroup
    v-for="control in controls"
    :key="control.side"
    :config="{
      ...control.position,
      visible,
      listening: interactive && visible,
    }"
    @pointerenter="setHovered(control.side, $event)"
    @pointerleave="setHovered(undefined, $event)"
    @pointerdown="begin(control, $event)"
  >
    <KonvaCircle
      :config="{
        radius: metrics.radius,
        fill: palette.panel,
        stroke: hoveredSide === control.side ? palette.accent : palette.border,
        strokeWidth: metrics.strokeWidth,
        hitStrokeWidth: metrics.hitExtension,
      }"
    />
    <KonvaArrow
      :config="{
        points: points(control.side),
        stroke: hoveredSide === control.side ? palette.accent : palette.foregroundSecondary,
        fill: hoveredSide === control.side ? palette.accent : palette.foregroundSecondary,
        strokeWidth: 1.35 / zoom,
        pointerLength: 3.5 / zoom,
        pointerWidth: 3.5 / zoom,
        lineCap: 'round',
        lineJoin: 'round',
        listening: false,
      }"
    />
  </KonvaGroup>
</template>
