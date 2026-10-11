<script setup lang="ts">
import type Konva from "konva";
import { shallowRef } from "vue";
import { Group as KonvaGroup, Rect as KonvaRect, Text as KonvaText, Line as KonvaLine } from "vue-konva/core";

import { SECTION_HEADER_HEIGHT, type ProjectedSection } from "../../document/structure-scene";
import type { SectionResizeCorner } from "../interaction/structure-gesture-preview";
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
  dragStart: [event: Konva.KonvaEventObject<DragEvent>];
  dragMove: [event: Konva.KonvaEventObject<DragEvent>];
  gestureEnd: [];
  pointerDown: [event: Konva.KonvaEventObject<PointerEvent>];
  rename: [];
  resizeStart: [corner: SectionResizeCorner, event: Konva.KonvaEventObject<DragEvent>];
  resizeMove: [event: Konva.KonvaEventObject<DragEvent>];
}>();
const hovered = shallowRef(false);
const width = () => props.section.bounds.right - props.section.bounds.left;
const height = () => props.section.bounds.bottom - props.section.bounds.top;
const displayCollapsed = () => props.section.layout.collapsed && !props.section.temporarilyExpanded;

function rename() {
  if (props.interactive && !props.section.locked) emit("rename");
}
</script>

<template>
  <KonvaGroup
    :config="{
      id: `canvas-section-chrome-${section.section.id}`,
      structureNodeId: section.section.id,
      x: section.bounds.left,
      y: section.bounds.top,
      draggable: interactive && !section.locked,
    }"
    @dragstart="emit('dragStart', $event)"
    @dragmove="emit('dragMove', $event)"
    @dragend="emit('gestureEnd')"
    @pointerenter="hovered = true"
    @pointerleave="hovered = false"
    @pointerdown="emit('pointerDown', $event)"
  >
    <KonvaRect
      :config="{
        width: width(),
        height: displayCollapsed() ? SECTION_HEADER_HEIGHT : height(),
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
        height: SECTION_HEADER_HEIGHT,
        fill: palette.panel,
        opacity: 0.94,
        cornerRadius: displayCollapsed() ? 8 : [8, 8, 0, 0],
        perfectDrawEnabled: false,
      }"
      @dblclick="rename"
    />
    <KonvaText
      :config="{
        x: 12,
        y: 0,
        height: SECTION_HEADER_HEIGHT,
        verticalAlign: 'middle',
        width: width() - 88,
        text: section.section.title,
        fill: palette.foreground,
        fontFamily: palette.typography.fontFamily,
        fontSize: palette.typography.labelSize,
        fontStyle: '500',
        ellipsis: true,
        wrap: 'none',
      }"
      @dblclick="rename"
    />
    <KonvaText
      :config="{
        x: width() - 68,
        y: 0,
        height: SECTION_HEADER_HEIGHT,
        verticalAlign: 'middle',
        width: 32,
        align: 'right',
        text: String(section.contentCount),
        fill: palette.foregroundSecondary,
        fontFamily: palette.typography.fontFamily,
        fontSize: palette.typography.captionSize,
        listening: false,
      }"
    />
    <KonvaGroup
      :config="{ x: width() - 32, y: 10 }"
      @click="interactive && emit('collapse', !displayCollapsed())"
    >
      <KonvaRect :config="{ width: 20, height: 20, fill: 'rgba(0,0,0,0.001)', cornerRadius: 4 }" />
      <KonvaLine
        :config="{
          points: displayCollapsed() ? [8, 5, 13, 10, 8, 15] : [5, 8, 10, 13, 15, 8],
          stroke: palette.foregroundSecondary,
          strokeWidth: 1.6,
          lineCap: 'round',
          lineJoin: 'round',
          listening: false,
        }"
      />
    </KonvaGroup>
    <template v-if="selected && !section.locked && !displayCollapsed()">
      <CanvasSectionResizeHandles
        :interactive="interactive"
        :palette="palette"
        :section="section"
        :zoom="zoom"
        @resize-start="(corner, event) => emit('resizeStart', corner, event)"
        @resize-move="emit('resizeMove', $event)"
        @gesture-end="emit('gestureEnd')"
      />
    </template>
  </KonvaGroup>
</template>
