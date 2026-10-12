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
  dragStart: [nodeId: string, event: Konva.KonvaEventObject<DragEvent>];
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
  onDblclick: activate,
  onDragstart: beginDrag,
  onDragmove: previewDrag,
  onDragend: finishDrag,
  onPointerdown: pressNode,
}));

function moveToLayer(event: Konva.KonvaEventObject<DragEvent>, layerId: string) {
  const layer = event.target.getStage()?.findOne<Konva.Layer>(`#${layerId}`);
  if (layer) event.target.moveTo(layer);
}

function activate() {
  if (props.interactive) emit("activate", props.sceneNode.node.id);
}

function beginDrag(event: Konva.KonvaEventObject<DragEvent>) {
  moveToLayer(event, "canvas-drag-layer");
  emit("dragStart", props.sceneNode.node.id, event);
}

function previewDrag(event: Konva.KonvaEventObject<DragEvent>) {
  emit("dragMove", props.sceneNode.node.id, event);
}

function finishDrag(event: Konva.KonvaEventObject<DragEvent>) {
  moveToLayer(event, "canvas-node-layer");
  emit("dragEnd", props.sceneNode.node.id);
}

function pressNode(event: Konva.KonvaEventObject<PointerEvent>) {
  emit("pointerDown", props.sceneNode.node.id, event);
}
</script>

<template>
  <KonvaGroup :config="groupConfig">
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
