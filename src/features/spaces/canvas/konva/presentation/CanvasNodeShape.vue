<script setup lang="ts">
import type Konva from "konva";
import { computed } from "vue";
import { Group as KonvaGroup, Rect as KonvaRect } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";

import { SECTION_HEADER_SCREEN_HEIGHT } from "../../contract/section-metrics";
import type { CanvasSceneNode } from "../scene/scene-projection";
import type { CanvasPalette } from "./use-canvas-palette";
const props = defineProps<{
  interactive: boolean;
  item: CanvasSceneNode;
  palette: CanvasPalette;
  zoom: number;
  selected: boolean;
}>();
const emit = defineEmits<{
  activate: [id: string];
  dragEnd: [id: string];
  dragStart: [id: string];
  dragMove: [id: string, event: Konva.KonvaEventObject<DragEvent>];
  pointerDown: [id: string, event: Konva.KonvaEventObject<PointerEvent>];
}>();
const frame = computed(() => props.item.layout.frame);
const section = computed(() => props.item.node.data.kind === "section");
const group = computed(() => props.item.node.data.kind === "group");
const collapsed = computed(() => props.item.node.data.kind === "section" && props.item.node.data.collapsed);
const color = computed(() => {
  const appearance = props.item.layout.appearance;
  return (
    appearance?.customFill ??
    (appearance?.fill ? props.palette.noteFill[appearance.fill] : props.palette.surface)
  );
});
const config = computed(() => ({
  id: `canvas-node-${props.item.node.id}`,
  name: "canvas-node",
  nodeId: props.item.node.id,
  x: frame.value.x,
  y: frame.value.y,
  rotation: frame.value.rotation,
  draggable: props.interactive && props.item.interaction.movable,
}));
</script>
<template>
  <KonvaGroup
    :config="config"
    @dblclick="interactive && emit('activate', item.node.id)"
    @dragstart="emit('dragStart', item.node.id)"
    @dragend="emit('dragEnd', item.node.id)"
    @dragmove="emit('dragMove', item.node.id, $event)"
    @pointerdown="emit('pointerDown', item.node.id, $event)"
  >
    <KonvaRect
      :config="{
        width: frame.width,
        height: frame.height,
        fill: section ? color : 'transparent',
        opacity: section ? 0.12 : 1,
        listening: section ? collapsed : !group || selected,
        strokeEnabled: false,
        cornerRadius: section ? 6 : 0,
        perfectDrawEnabled: false,
      }"
    />
    <KonvaRect
      v-if="section"
      :config="{
        width: frame.width,
        height: frame.height,
        listening: false,
        stroke: selected ? palette.accent : palette.border,
        strokeWidth: 1 / zoom,
        strokeScaleEnabled: false,
        cornerRadius: 6,
        perfectDrawEnabled: false,
      }"
    />
    <KonvaRect
      v-if="section && !collapsed"
      :config="{
        width: frame.width,
        height: Math.min(frame.height, SECTION_HEADER_SCREEN_HEIGHT / zoom),
        fill: 'transparent',
        perfectDrawEnabled: false,
      }"
    />
  </KonvaGroup>
</template>
