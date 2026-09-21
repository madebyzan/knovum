<script setup lang="ts">
import type Konva from "konva";
import { computed, nextTick, onMounted, useTemplateRef } from "vue";
import { Layer as KonvaLayer } from "vue-konva/core";
import type { VueKonvaRef } from "vue-konva/core";

import type { ProjectedGroup, ProjectedSection } from "../../document/structure-scene";
import CanvasGroupChrome from "./CanvasGroupChrome.vue";
import CanvasSectionChrome from "./CanvasSectionChrome.vue";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  groups: readonly ProjectedGroup[];
  groupScopePath: readonly string[];
  interactive: boolean;
  palette: CanvasPalette;
  sections: readonly ProjectedSection[];
  selectedIds: ReadonlySet<string>;
  dropTargetId: string | undefined;
  zoom: number;
}>();
const emit = defineEmits<{
  collapse: [structureNodeId: string, collapsed: boolean];
  enterGroup: [structureNodeId: string];
  move: [structureNodeId: string, delta: { x: number; y: number }];
  movePreview: [structureNodeId: string, delta: { x: number; y: number } | undefined];
  rename: [structureNodeId: string];
  resize: [structureNodeId: string, frame: { x: number; y: number; width: number; height: number }];
  select: [structureNodeId: string, event: PointerEvent];
  transform: [
    structureNodeId: string,
    transform: { anchor: { x: number; y: number }; rotationDelta: number; scaleFactor: number },
  ];
}>();
const selectedGroups = computed(() => {
  const scopeId = props.groupScopePath.at(-1) ?? null;
  return props.groups.filter(
    ({ group, visible }) => visible && group.parentId === scopeId && props.selectedIds.has(group.id),
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

onMounted(() => void nextTick(syncLayerOrder));
</script>

<template>
  <KonvaLayer ref="sectionLayer" :config="{ id: 'canvas-section-interaction-layer' }">
    <CanvasSectionChrome
      v-for="section in sections"
      :key="section.section.id"
      :interactive="interactive"
      :palette="palette"
      :section="section"
      :selected="selectedIds.has(section.section.id)"
      :drop-target="dropTargetId === section.section.id"
      :zoom="zoom"
      @collapse="emit('collapse', section.section.id, $event)"
      @move="emit('move', section.section.id, $event)"
      @rename="emit('rename', section.section.id)"
      @resize="emit('resize', section.section.id, $event)"
      @select="emit('select', section.section.id, $event)"
    />
  </KonvaLayer>
  <KonvaLayer ref="groupLayer" :config="{ id: 'canvas-group-interaction-layer' }">
    <CanvasGroupChrome
      v-for="group in selectedGroups"
      :key="group.group.id"
      :group="group"
      :interactive="interactive"
      :palette="palette"
      :zoom="zoom"
      @enter="emit('enterGroup', group.group.id)"
      @move="emit('move', group.group.id, $event)"
      @move-preview="emit('movePreview', group.group.id, $event)"
      @rename="emit('rename', group.group.id)"
      @select="emit('select', group.group.id, $event)"
      @transform="emit('transform', group.group.id, $event)"
    />
  </KonvaLayer>
</template>
