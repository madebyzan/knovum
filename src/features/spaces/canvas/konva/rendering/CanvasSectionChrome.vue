<script setup lang="ts">
import type Konva from "konva";
import { computed, shallowRef } from "vue";
import { Group as KonvaGroup, Line as KonvaLine, Rect as KonvaRect, Text as KonvaText } from "vue-konva/core";

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
const collapseHovered = shallowRef(false);
const width = computed(() => props.section.bounds.right - props.section.bounds.left);
const height = computed(() => props.section.bounds.bottom - props.section.bounds.top);
const displayCollapsed = computed(() => props.section.layout.collapsed && !props.section.temporarilyExpanded);
const countText = computed(() => String(props.section.contentCount));
const badgeWidth = computed(() => Math.max(20, countText.value.length * 7 + 12));

const sectionFill = computed(() => {
  const appearance = props.section.layout.appearance;
  if (appearance?.customFill) return appearance.customFill;
  if (appearance?.fill) return props.palette.contentFill[appearance.fill];
  return props.palette.panel;
});

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
        cornerRadius: 12,
        listening: false,
      }"
    />
    <KonvaRect
      :config="{
        width,
        height: SECTION_HEADER_HEIGHT,
        fill: sectionFill,
        opacity: 0.75,
        cornerRadius: displayCollapsed ? 12 : [12, 12, 0, 0],
        perfectDrawEnabled: false,
        onDblclick: rename,
      }"
    />
    <KonvaLine
      v-if="!displayCollapsed"
      :config="{
        points: [0, SECTION_HEADER_HEIGHT, width, SECTION_HEADER_HEIGHT],
        stroke: hovered ? palette.borderStrong : palette.border,
        strokeWidth: 1 / zoom,
        opacity: 0.5,
        listening: false,
      }"
    />
    <KonvaRect
      :config="{
        width,
        height: displayCollapsed ? SECTION_HEADER_HEIGHT : height,
        stroke: selected || dropAction ? palette.accent : hovered ? palette.borderStrong : palette.border,
        strokeWidth: (selected || dropAction ? 2 : 1) / zoom,
        fillEnabled: false,
        dash:
          dropAction || section.layout.appearance?.borderStyle === 'dashed'
            ? [7 / zoom, 5 / zoom]
            : undefined,
        hitStrokeWidth: 12 / zoom,
        cornerRadius: 12,
        perfectDrawEnabled: false,
      }"
    />
    <CanvasSectionDropFeedback v-if="dropAction" :action="dropAction" :palette="palette" :zoom="zoom" />
    <KonvaGroup :config="{ x: 14, y: 13, listening: false }">
      <KonvaLine
        :config="{
          points: [0, 0, 14, 0, 14, 14, 0, 14, 0, 0],
          stroke: palette.foregroundSecondary,
          strokeWidth: 1.2,
          lineJoin: 'round',
        }"
      />
      <KonvaLine
        :config="{ points: [0, 4.5, 14, 4.5], stroke: palette.foregroundSecondary, strokeWidth: 1.2 }"
      />
    </KonvaGroup>
    <KonvaText
      :config="{
        x: 36,
        y: 0,
        height: SECTION_HEADER_HEIGHT,
        verticalAlign: 'middle',
        width: Math.max(0, width - 36 - badgeWidth - 44),
        text: section.section.title,
        fill: palette.foreground,
        fontFamily: palette.typography.fontFamily,
        fontSize: palette.typography.labelSize,
        fontStyle: '600',
        ellipsis: true,
        wrap: 'none',
        onDblclick: rename,
      }"
    />
    <KonvaGroup :config="{ x: width - 34 - badgeWidth, y: 11, listening: false }">
      <KonvaRect
        :config="{ width: badgeWidth, height: 18, fill: palette.border, cornerRadius: 9, opacity: 0.8 }"
      />
      <KonvaText
        :config="{
          width: badgeWidth,
          height: 18,
          text: countText,
          align: 'center',
          verticalAlign: 'middle',
          fill: palette.foregroundSecondary,
          fontFamily: palette.typography.fontFamily,
          fontSize: Math.max(10, palette.typography.captionSize - 1),
          fontStyle: '500',
        }"
      />
    </KonvaGroup>
    <KonvaGroup
      :config="{
        x: width - 30,
        y: 9,
        onClick: toggleCollapsed,
        onPointerenter: () => (collapseHovered = true),
        onPointerleave: () => (collapseHovered = false),
      }"
    >
      <KonvaRect
        :config="{
          width: 22,
          height: 22,
          fill: collapseHovered ? palette.border : 'rgba(0,0,0,0.001)',
          cornerRadius: 6,
          opacity: collapseHovered ? 0.6 : 1,
        }"
      />
      <KonvaLine
        :config="{
          points: displayCollapsed ? [8, 6, 13, 11, 8, 16] : [6, 8, 11, 13, 16, 8],
          stroke: collapseHovered ? palette.foreground : palette.foregroundSecondary,
          strokeWidth: 1.5,
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
