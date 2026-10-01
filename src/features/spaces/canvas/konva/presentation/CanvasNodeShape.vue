<script setup lang="ts">
import type Konva from "konva";
import { computed } from "vue";
import { Group as KonvaGroup, Rect as KonvaRect } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";

import type { CanvasSceneNode } from "../scene/canvas-scene";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  interactive: boolean;
  sceneNode: CanvasSceneNode;
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

const frame = computed(() => props.sceneNode.layout.frame);
const groupConfig = computed(() => ({
  id: `canvas-node-${props.sceneNode.node.id}`,
  name: "canvas-node",
  nodeId: props.sceneNode.node.id,
  x: frame.value.x,
  y: frame.value.y,
  rotation: frame.value.rotation,
  draggable: props.interactive && props.sceneNode.interaction.movable,
}));

function moveToLayer(event: Konva.KonvaEventObject<DragEvent>, layerId: string) {
  const layer = event.target.getStage()?.findOne<Konva.Layer>(`#${layerId}`);
  if (layer) event.target.moveTo(layer);
}

function beginDrag(event: Konva.KonvaEventObject<DragEvent>) {
  moveToLayer(event, "canvas-drag-layer");
  emit("dragStart", props.sceneNode.node.id);
}

function finishDrag(event: Konva.KonvaEventObject<DragEvent>) {
  moveToLayer(event, "canvas-node-layer");
  emit("dragEnd", props.sceneNode.node.id);
}
</script>

<template>
  <KonvaGroup
    :config="groupConfig"
    @dblclick="interactive && emit('activate', sceneNode.node.id)"
    @dragend="finishDrag"
    @dragmove="emit('dragMove', sceneNode.node.id, $event)"
    @dragstart="beginDrag"
    @pointerdown="emit('pointerDown', sceneNode.node.id, $event)"
  >
    <KonvaRect
      :config="{
        width: frame.width,
        height: frame.height,
        fill: 'transparent',
        cornerRadius: 10,
        perfectDrawEnabled: false,
      }"
    />
  </KonvaGroup>
</template>
