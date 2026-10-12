<script setup lang="ts">
import type Konva from "konva";
import { computed, shallowRef } from "vue";
import { Group as KonvaGroup, Rect as KonvaRect, Text as KonvaText, Line as KonvaLine } from "vue-konva/core";

import { SECTION_HEADER_HEIGHT } from "../../../model/structure/section-metrics";
import type { SectionDropPreview } from "../../../structure/section-drop-reparent";
import type { ProjectedSection } from "../../document/structure-scene";
import type { SectionResizeCorner } from "../interaction/structure-gesture-preview";
import CanvasSectionDropFeedback from "./CanvasSectionDropFeedback.vue";
import CanvasSectionResizeHandles from "./CanvasSectionResizeHandles.vue";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  interactive: boolean;
  palette: CanvasPalette;
  section: ProjectedSection;
  selected: boolean;
  dropAction: SectionDropPreview["kind"] | undefined;
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
const width = computed(() => props.section.bounds.right - props.section.bounds.left);
const height = computed(() => props.section.bounds.bottom - props.section.bounds.top);
const displayCollapsed = computed(() => props.section.layout.collapsed && !props.section.temporarilyExpanded);

function rename() {
  if (props.interactive && !props.section.locked) emit("rename");
}

function toggleCollapsed() {
  if (props.interactive) emit("collapse", !displayCollapsed.value);
}

const chromeConfig = computed(() => ({
  id: `canvas-section-chrome-${props.section.section.id}`,
  structureNodeId: props.section.section.id,
  x: props.section.bounds.left,
  y: props.section.bounds.top,
  draggable: props.interactive && !props.section.locked,
  onDragstart: (event: Konva.KonvaEventObject<DragEvent>) => emit("dragStart", event),
  onDragmove: (event: Konva.KonvaEventObject<DragEvent>) => emit("dragMove", event),
  onDragend: () => emit("gestureEnd"),
  onPointerenter: () => (hovered.value = true),
  onPointerleave: () => (hovered.value = false),
  onPointerdown: (event: Konva.KonvaEventObject<PointerEvent>) => emit("pointerDown", event),
}));
</script>

<template>
  <KonvaGroup :config="chromeConfig">
    <KonvaRect
      v-if="dropAction"
      :config="{
        width,
        height: displayCollapsed ? SECTION_HEADER_HEIGHT : height,
        fill: palette.accent,
        opacity: 0.08,
        cornerRadius: 8,
        listening: false,
      }"
    />
    <KonvaRect
      :config="{
        width: width,
        height: SECTION_HEADER_HEIGHT,
        fill: palette.panel,
        opacity: 0.94,
        cornerRadius: displayCollapsed ? 8 : [8, 8, 0, 0],
        perfectDrawEnabled: false,
        onDblclick: rename,
      }"
    />
    <KonvaRect
      :config="{
        width: width,
        height: displayCollapsed ? SECTION_HEADER_HEIGHT : height,
        stroke: selected || dropAction ? palette.accent : hovered ? palette.borderStrong : palette.border,
        strokeWidth: (dropAction ? 2 : 1) / zoom,
        fillEnabled: false,
        dash:
          dropAction || section.layout.appearance?.borderStyle === 'dashed'
            ? [7 / zoom, 5 / zoom]
            : undefined,
        hitStrokeWidth: 12 / zoom,
        cornerRadius: 8,
        perfectDrawEnabled: false,
      }"
    />
    <CanvasSectionDropFeedback v-if="dropAction" :action="dropAction" :palette="palette" :zoom="zoom" />
    <KonvaText
      :config="{
        x: 12,
        y: 0,
        height: SECTION_HEADER_HEIGHT,
        verticalAlign: 'middle',
        width: width - 88,
        text: section.section.title,
        fill: palette.foreground,
        fontFamily: palette.typography.fontFamily,
        fontSize: palette.typography.labelSize,
        fontStyle: '500',
        ellipsis: true,
        wrap: 'none',
        onDblclick: rename,
      }"
    />
    <KonvaText
      :config="{
        x: width - 68,
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
    <KonvaGroup :config="{ x: width - 32, y: 10, onClick: toggleCollapsed }">
      <KonvaRect :config="{ width: 20, height: 20, fill: 'rgba(0,0,0,0.001)', cornerRadius: 4 }" />
      <KonvaLine
        :config="{
          points: displayCollapsed ? [8, 5, 13, 10, 8, 15] : [5, 8, 10, 13, 15, 8],
          stroke: palette.foregroundSecondary,
          strokeWidth: 1.6,
          lineCap: 'round',
          lineJoin: 'round',
          listening: false,
        }"
      />
    </KonvaGroup>
    <template v-if="selected && !section.locked && !displayCollapsed">
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
