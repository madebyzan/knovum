<script setup lang="ts">
import type Konva from "konva";
import { computed } from "vue";
import { Group as KonvaGroup, Rect as KonvaRect } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";

import type { CanvasSceneNode } from "../scene/scene-projection";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  interactive: boolean;
  item: CanvasSceneNode;
  palette: CanvasPalette;
  zoom: number;
}>();
const emit = defineEmits<{
  activate: [nodeId: string];
  dragEnd: [nodeId: string];
  dragMove: [nodeId: string, event: Konva.KonvaEventObject<DragEvent>];
  dragStart: [nodeId: string];
  pointerDown: [nodeId: string, event: Konva.KonvaEventObject<PointerEvent>];
}>();

const frame = computed(() => props.item.layout.frame);
const isSection = computed(() => props.item.node.data.kind === "section");
const appearanceColor = computed(() => {
  const appearance = props.item.layout.appearance;
  if (appearance?.customFill) return appearance.customFill;
  const fill = appearance?.fill;
  return fill ? props.palette.noteFill[fill] : undefined;
});
const groupConfig = computed(() => ({
  id: `canvas-node-${props.item.node.id}`,
  name: "canvas-node",
  nodeId: props.item.node.id,
  x: frame.value.x,
  y: frame.value.y,
  rotation: frame.value.rotation,
  draggable: props.interactive && props.item.interaction.movable,
}));

function moveToLayer(event: Konva.KonvaEventObject<DragEvent>, layerId: string) {
  const layer = event.target.getStage()?.findOne<Konva.Layer>(`#${layerId}`);
  if (layer) event.target.moveTo(layer);
}

function beginDrag(event: Konva.KonvaEventObject<DragEvent>) {
  moveToLayer(event, "canvas-drag-layer");
  emit("dragStart", props.item.node.id);
}

function finishDrag(event: Konva.KonvaEventObject<DragEvent>) {
  moveToLayer(event, "canvas-node-layer");
  emit("dragEnd", props.item.node.id);
}
</script>

<template>
  <KonvaGroup
    :config="groupConfig"
    @dblclick="interactive && emit('activate', item.node.id)"
    @dragend="finishDrag"
    @dragmove="emit('dragMove', item.node.id, $event)"
    @dragstart="beginDrag"
    @pointerdown="emit('pointerDown', item.node.id, $event)"
  >
    <KonvaRect
      :config="{
        width: frame.width,
        height: frame.height,
        fill: isSection ? (appearanceColor ?? palette.surface) : 'transparent',
        opacity: isSection && appearanceColor ? 0.14 : 1,
        stroke: appearanceColor ?? palette.border,
        strokeEnabled: isSection,
        strokeWidth: 1 / zoom,
        cornerRadius: isSection ? 8 : 10,
        dash: isSection ? [7 / zoom, 5 / zoom] : undefined,
        perfectDrawEnabled: false,
      }"
    />
  </KonvaGroup>
</template>
