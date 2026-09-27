<script setup lang="ts">
import type Konva from "konva";
import { shallowRef } from "vue";
import { Group as KonvaGroup, Rect as KonvaRect, Text as KonvaText } from "vue-konva/core";

import type { ProjectedSection } from "../../document/structure-scene";
import CanvasSectionResizeHandles from "./CanvasSectionResizeHandles.vue";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  interactive: boolean;
  palette: CanvasPalette;
  section: ProjectedSection;
  selected: boolean;
  dropTarget: boolean;
  zoom: number;
}>();
const emit = defineEmits<{
  collapse: [collapsed: boolean];
  move: [delta: { x: number; y: number }];
  rename: [];
  resize: [frame: { x: number; y: number; width: number; height: number }];
  select: [event: PointerEvent];
}>();
let moveOrigin: { x: number; y: number } | undefined;
const hovered = shallowRef(false);
const width = () => props.section.bounds.right - props.section.bounds.left;
const height = () => props.section.bounds.bottom - props.section.bounds.top;
const displayCollapsed = () => props.section.layout.collapsed && !props.section.temporarilyExpanded;

function beginMove(event: Konva.KonvaEventObject<DragEvent>) {
  moveOrigin = { x: event.target.x(), y: event.target.y() };
}

function finishMove(event: Konva.KonvaEventObject<DragEvent>) {
  if (!moveOrigin) return;
  const delta = { x: event.target.x() - moveOrigin.x, y: event.target.y() - moveOrigin.y };
  event.target.position(moveOrigin);
  moveOrigin = undefined;
  if (delta.x || delta.y) emit("move", delta);
}

function select(event: Konva.KonvaEventObject<PointerEvent>) {
  event.cancelBubble = true;
  emit("select", event.evt);
}

function rename() {
  if (props.interactive && !props.section.locked) emit("rename");
}
</script>

<template>
  <KonvaGroup
    :config="{
      id: `canvas-section-chrome-${section.section.id}`,
      x: section.bounds.left,
      y: section.bounds.top,
      draggable: interactive && !section.locked,
    }"
    @dragstart="beginMove"
    @dragend="finishMove"
    @pointerenter="hovered = true"
    @pointerleave="hovered = false"
    @pointerdown="select"
  >
    <KonvaRect
      :config="{
        width: width(),
        height: displayCollapsed() ? 40 : height(),
        stroke: selected || dropTarget ? palette.accent : hovered ? palette.borderStrong : palette.border,
        strokeWidth: (selected || dropTarget ? 2 : 1) / zoom,
        dash:
          dropTarget || section.layout.appearance?.borderStyle === 'dashed'
            ? [7 / zoom, 5 / zoom]
            : undefined,
        opacity: dropTarget ? 0.72 : 1,
        hitStrokeWidth: 12 / zoom,
        cornerRadius: 8,
        perfectDrawEnabled: false,
      }"
    />
    <KonvaRect
      :config="{
        width: width(),
        height: 40,
        fill: palette.surface,
        opacity: 0.94,
        cornerRadius: displayCollapsed() ? 8 : [8, 8, 0, 0],
        perfectDrawEnabled: false,
      }"
      @dblclick="rename"
    />
    <KonvaText
      :config="{
        x: 12,
        y: 12,
        width: width() - 88,
        text: section.section.title,
        fill: palette.foreground,
        fontSize: 13,
        fontStyle: '600',
        ellipsis: true,
        wrap: 'none',
      }"
      @dblclick="rename"
    />
    <KonvaText
      :config="{
        x: width() - 68,
        y: 13,
        width: 32,
        align: 'right',
        text: String(section.contentCount),
        fill: palette.foregroundSecondary,
        fontSize: 11,
        listening: false,
      }"
    />
    <KonvaText
      :config="{
        x: width() - 32,
        y: 10,
        width: 20,
        height: 20,
        align: 'center',
        verticalAlign: 'middle',
        text: displayCollapsed() ? '›' : '⌄',
        fill: palette.foregroundSecondary,
        fontSize: 15,
      }"
      @pointerdown="$event.cancelBubble = true"
      @click="emit('collapse', !displayCollapsed())"
    />
    <template v-if="selected && !section.locked && !displayCollapsed()">
      <CanvasSectionResizeHandles
        :interactive="interactive"
        :palette="palette"
        :section="section"
        :zoom="zoom"
        @resize="emit('resize', $event)"
      />
    </template>
  </KonvaGroup>
</template>
