<script setup lang="ts">
import type Konva from "konva";
import { computed, watch, useTemplateRef } from "vue";
import { Layer as KonvaLayer } from "vue-konva/core";
import type { VueKonvaRef } from "vue-konva/core";

import type { ProjectedGroup, ProjectedSection } from "../../document/structure-scene";
import type { SectionResizeCorner } from "../interaction/structure-gesture-preview";
import CanvasGroupChrome from "./CanvasGroupChrome.vue";
import CanvasSectionChrome from "./CanvasSectionChrome.vue";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  groups: readonly ProjectedGroup[];
  groupScopePath: readonly string[];
  interactive: boolean;
  selectionChromeVisible: boolean;
  groupTransformEnabled: boolean;
  palette: CanvasPalette;
  sections: readonly ProjectedSection[];
  selectedIds: ReadonlySet<string>;
  dropTargetId: string | undefined;
  zoom: number;
}>();
const emit = defineEmits<{
  collapse: [structureNodeId: string, collapsed: boolean];
  enterGroup: [structureNodeId: string];
  dragStart: [structureNodeId: string, event: Konva.KonvaEventObject<DragEvent>];
  dragMove: [structureNodeId: string, event: Konva.KonvaEventObject<DragEvent>];
  gestureEnd: [];
  rename: [structureNodeId: string];
  resizeStart: [
    structureNodeId: string,
    corner: SectionResizeCorner,
    event: Konva.KonvaEventObject<DragEvent>,
  ];
  resizeMove: [event: Konva.KonvaEventObject<DragEvent>];
  pointerDown: [structureNodeId: string, event: Konva.KonvaEventObject<PointerEvent>];
  transformStart: [structureNodeId: string, event: Konva.KonvaEventObject<DragEvent>];
  transform: [transform: { anchor: { x: number; y: number }; rotationDelta: number; scaleFactor: number }];
}>();
const selectedGroups = computed(() => {
  const scopeId = props.groupScopePath.at(-1) ?? null;
  return props.groups.filter(
    ({ group, visible }) =>
      visible &&
      props.selectedIds.has(group.id) &&
      (group.parentId === scopeId ||
        (scopeId === null && props.sections.some(({ section }) => section.id === group.parentId))),
  );
});
const sectionLayer = useTemplateRef<VueKonvaRef<Konva.Layer>>("sectionLayer");
const groupLayer = useTemplateRef<VueKonvaRef<Konva.Layer>>("groupLayer");

function syncLayerOrder() {
  const sectionCanvas = sectionLayer.value?.getNode().getNativeCanvasElement();
  const groupCanvas = groupLayer.value?.getNode().getNativeCanvasElement();
  if (sectionCanvas) sectionCanvas.style.zIndex = "15";
  if (groupCanvas) {
    groupCanvas.style.zIndex = "16";
    groupCanvas.dataset.thumbnailExclude = "";
  }
}

watch([sectionLayer, groupLayer], syncLayerOrder, { flush: "post" });
</script>

<template>
  <KonvaLayer
    v-if="sections.length"
    ref="sectionLayer"
    :config="{ id: 'canvas-section-interaction-layer', listening: interactive }"
  >
    <CanvasSectionChrome
      v-for="section in sections"
      :key="section.section.id"
      :interactive="interactive"
      :palette="palette"
      :section="section"
      :selected="selectionChromeVisible && selectedIds.has(section.section.id)"
      :drop-target="dropTargetId === section.section.id"
      :zoom="zoom"
      @collapse="emit('collapse', section.section.id, $event)"
      @drag-start="emit('dragStart', section.section.id, $event)"
      @drag-move="emit('dragMove', section.section.id, $event)"
      @gesture-end="emit('gestureEnd')"
      @rename="emit('rename', section.section.id)"
      @resize-start="(corner, event) => emit('resizeStart', section.section.id, corner, event)"
      @resize-move="emit('resizeMove', $event)"
      @pointer-down="emit('pointerDown', section.section.id, $event)"
    />
  </KonvaLayer>
  <KonvaLayer
    v-if="selectedGroups.length"
    ref="groupLayer"
    :config="{ id: 'canvas-group-interaction-layer', listening: interactive }"
  >
    <CanvasGroupChrome
      v-for="group in selectedGroups"
      :key="group.group.id"
      :group="group"
      :interactive="interactive"
      :handles-visible="selectionChromeVisible && groupTransformEnabled"
      :palette="palette"
      :zoom="zoom"
      @enter="emit('enterGroup', group.group.id)"
      @drag-start="emit('dragStart', group.group.id, $event)"
      @drag-move="emit('dragMove', group.group.id, $event)"
      @gesture-end="emit('gestureEnd')"
      @rename="emit('rename', group.group.id)"
      @pointer-down="emit('pointerDown', group.group.id, $event)"
      @transform-start="emit('transformStart', group.group.id, $event)"
      @transform="emit('transform', $event)"
    />
  </KonvaLayer>
</template>
