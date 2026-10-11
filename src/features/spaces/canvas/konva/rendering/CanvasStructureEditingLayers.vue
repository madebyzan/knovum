<script setup lang="ts">
import type Konva from "konva";

import type { CanvasSurfaceProps } from "../../contracts/canvas-surface";
import type { SectionResizeCorner } from "../interaction/structure-gesture-preview";
import type { useKonvaCanvasRuntime } from "../use-konva-canvas-runtime";
import CanvasStructureInteractionLayer from "./CanvasStructureInteractionLayer.vue";

const props = defineProps<{
  runtime: ReturnType<typeof useKonvaCanvasRuntime>;
  interactive: boolean;
  selectionChromeVisible: boolean;
  groupScopePath: CanvasSurfaceProps["groupScopePath"];
}>();
function beginSectionResize(
  structureNodeId: string,
  corner: SectionResizeCorner,
  event: Konva.KonvaEventObject<DragEvent>,
) {
  props.runtime.pointer.markSelectionDragged();
  props.runtime.structureGestures.beginResize(
    structureNodeId,
    corner,
    event.target,
    props.runtime.pointer.pressedCanvasPoint.value!,
  );
}

function beginGroupTransform(structureNodeId: string, event: Konva.KonvaEventObject<DragEvent>) {
  props.runtime.pointer.markSelectionDragged();
  props.runtime.structureGestures.beginTransform(structureNodeId, event.target);
}
</script>

<template>
  <CanvasStructureInteractionLayer
    :groups="runtime.structure.structureScene.value.groups"
    :group-scope-path="groupScopePath"
    :group-transform-enabled="runtime.canTransformGroups.value"
    :interactive="interactive"
    :selection-chrome-visible="selectionChromeVisible"
    :palette="runtime.palette.value"
    :sections="runtime.visibility.visibleSections.value"
    :selected-ids="runtime.selection.structureNodeIds.value"
    :drop-preview="runtime.structureInteractions.dropPreview.value"
    :zoom="runtime.viewport.viewport.value.zoom"
    @collapse="runtime.structureInteractions.collapse"
    @enter-group="runtime.structureInteractions.enterGroup"
    @drag-start="runtime.nodeDrag.begin"
    @drag-move="(_id, event) => runtime.nodeDrag.move(event)"
    @gesture-end="runtime.finishDrag"
    @rename="runtime.structureInteractions.beginRename"
    @resize-start="beginSectionResize"
    @resize-move="runtime.structureGestures.resize"
    @pointer-down="runtime.pointer.structurePointerDown"
    @transform-start="beginGroupTransform"
    @transform="runtime.structureGestures.transform"
  />
</template>
