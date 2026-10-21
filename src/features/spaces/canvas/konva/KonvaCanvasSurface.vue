<script setup lang="ts">
import { computed } from "vue";
import { Stage as KonvaStage } from "vue-konva/core";

import type { CanvasNodeContentSlotProps, CanvasSurfaceProps } from "../contracts/canvas-surface";
import type { CanvasDocumentV1 } from "../schema/document";
import CanvasDomOverlay from "./rendering/CanvasDomOverlay.vue";
import CanvasGridSurface from "./rendering/CanvasGridSurface.vue";
import CanvasMarquee from "./rendering/CanvasMarquee.vue";
import CanvasSceneLayers from "./rendering/CanvasSceneLayers.vue";
import CanvasSectionCandidateFeedback from "./rendering/CanvasSectionCandidateFeedback.vue";
import CanvasStructureEditingLayers from "./rendering/CanvasStructureEditingLayers.vue";
import CanvasStructureHud from "./rendering/CanvasStructureHud.vue";
import CanvasStructureSectionsLayer from "./rendering/CanvasStructureSectionsLayer.vue";
import { useKonvaCanvasRuntime } from "./use-konva-canvas-runtime";

const props = defineProps<CanvasSurfaceProps>();
const emit = defineEmits<{ documentChange: [document: CanvasDocumentV1] }>();
defineSlots<{ "node-content"(props: CanvasNodeContentSlotProps): unknown }>();
const runtime = useKonvaCanvasRuntime(props, (document) => emit("documentChange", document));
const selectGesturesEnabled = computed(
  () =>
    props.interactionEnabled &&
    props.canCommitCanvasGestures &&
    props.tool === "select" &&
    !runtime.viewport.panning.value,
);
const isInteractive = computed(
  () =>
    selectGesturesEnabled.value &&
    !runtime.gestures.busy.value &&
    !runtime.capturesCanvasInteraction.value &&
    !runtime.viewport.spacePressed.value,
);
const selectionChromeVisible = computed(
  () =>
    selectGesturesEnabled.value &&
    !runtime.viewport.spacePressed.value &&
    runtime.directManipulation.value !== "drag",
);
const cursorClass = computed(() => {
  if (runtime.connectors.draft.value) return "cursor-crosshair";
  if (runtime.viewport.panning.value) return "cursor-grabbing";
  if (runtime.viewport.spacePressed.value || props.tool === "pan") return "cursor-grab";
  return ["note", "text", "section"].includes(props.tool) ? "cursor-crosshair" : "";
});
defineExpose({ flushViewport: runtime.viewport.flushViewport });
</script>

<template>
  <div
    ref="host"
    data-space-thumbnail-source
    data-editing-surface
    class="relative h-full min-h-0 w-full overflow-hidden outline-none"
    :class="cursorClass"
    tabindex="0"
    @contextmenu="interactionEnabled && runtime.contextMenu.pointer($event)"
    @keydown="
      interactionEnabled && !runtime.contextMenu.keyboard($event) && runtime.viewport.pressSpaceKey($event)
    "
  >
    <CanvasGridSurface
      v-if="renderingEnabled"
      :camera="runtime.viewport.camera"
      :palette="runtime.palette.value"
      :surface="runtime.viewport.surface"
      :zoom="runtime.viewport.zoom.value"
    />
    <!--
    The stage transform is written by the camera, not bound here: a reactive position would re-render this
    whole subtree on every frame of a pan and leave Konva to redraw one frame later than the DOM overlay.
    -->
    <KonvaStage
      ref="stage"
      :config="{
        visible: renderingEnabled,
        listening: renderingEnabled,
        width: runtime.viewport.surface.width,
        height: runtime.viewport.surface.height,
      }"
      @pointerdown="interactionEnabled && canCommitCanvasGestures && runtime.pointer.pointerDown($event)"
      @pointermove="interactionEnabled && canCommitCanvasGestures && runtime.connectors.pointerMove($event)"
      @pointercancel="runtime.pointer.pointerCancel($event.evt)"
      @pointerup="interactionEnabled && canCommitCanvasGestures && runtime.connectors.pointerUp($event)"
      @wheel="interactionEnabled && runtime.viewport.wheel($event)"
    >
      <CanvasStructureSectionsLayer
        :palette="runtime.palette.value"
        :sections="runtime.visibility.visibleSections.value"
        @error="events.onError"
      />
      <CanvasSceneLayers
        :interactive="isInteractive"
        :selection-chrome-visible="selectionChromeVisible"
        :connector-draft="runtime.connectors.draft.value"
        :feedback="runtime.feedback.value"
        :palette="runtime.palette.value"
        :layout-preview-by-node-id="runtime.scenePreview.layoutByNodeId"
        :relation-preview-by-id="runtime.scenePreview.relationById"
        :scene="runtime.scene.value"
        :selected-node-ids="runtime.selection.nodeIds.value"
        :selected-relation-ids="runtime.visibility.selectedRelationIds.value"
        :transform-policy="runtime.transformPolicy.value"
        :zoom="runtime.viewport.zoom.value"
        v-bind="runtime.stage.value ? { stage: runtime.stage.value } : {}"
        :visible-node-ids="runtime.visibility.visibleNodeIds.value"
        :visible-relation-ids="runtime.visibility.visibleRelationIds.value"
        @activate="runtime.activateNode"
        @connector-start="runtime.connectors.begin"
        @drag-end="runtime.finishDrag"
        @drag-move="(_id, event) => runtime.nodeDrag.move(event)"
        @drag-start="runtime.nodeDrag.beginNode"
        @pointer-down="runtime.pointer.nodePointerDown"
        @relation-pointer-down="runtime.selection.selectRelation"
        @resize-snap="runtime.gestures.controller.updateResizeGuides"
        @transform="runtime.gestures.controller.previewTransform($event)"
        @transform-end="runtime.finishTransform"
        @transform-start="runtime.gestures.beginTransform()"
      />
      <CanvasSectionCandidateFeedback
        :bounds="runtime.sectionCandidateBounds.value"
        :palette="runtime.palette.value"
        :zoom="runtime.viewport.zoom.value"
      />
      <CanvasStructureEditingLayers
        :runtime="runtime"
        :interactive="isInteractive"
        :selection-chrome-visible="selectionChromeVisible"
        :group-scope-path="groupScopePath"
      />
      <CanvasMarquee
        :bounds="runtime.selection.marqueeRect.value"
        :palette="runtime.palette.value"
        :zoom="runtime.viewport.zoom.value"
      />
      <CanvasMarquee
        :bounds="runtime.pointer.sectionDraft.value"
        :palette="runtime.palette.value"
        :zoom="runtime.viewport.zoom.value"
      />
    </KonvaStage>
    <Teleport v-if="runtime.stage.value" :to="runtime.stage.value.getContent()">
      <CanvasDomOverlay
        :camera="runtime.viewport.camera"
        :rendering-enabled="renderingEnabled"
        :interaction-enabled="
          interactionEnabled &&
          tool === 'select' &&
          !runtime.viewport.spacePressed.value &&
          !runtime.viewport.panning.value
        "
        :interactive-node-id="interactionEnabled ? runtime.interactiveNodeId.value : undefined"
        :layout-preview-by-node-id="runtime.scenePreview.layoutByNodeId"
        :scene="runtime.scene.value"
        :visible-node-ids="runtime.visibility.visibleContentNodeIds.value"
        :selected-node-ids="runtime.selection.nodeIds.value"
        :zoom="runtime.viewport.zoom.value"
      >
        <template #default="content">
          <slot name="node-content" v-bind="content" />
        </template>
      </CanvasDomOverlay>
    </Teleport>
    <CanvasStructureHud
      :camera="runtime.viewport.camera"
      :group-scope-titles="runtime.structure.groupScopeTitles.value"
      :rename-target="runtime.structureInteractions.renameTarget.value"
      :rename-error="runtime.structureInteractions.renameError.value"
      :zoom="runtime.viewport.zoom.value"
      @back="runtime.structureInteractions.exitGroup"
      @cancel-rename="runtime.structureInteractions.cancelRename"
      @save-rename="runtime.structureInteractions.commitRename"
    />
  </div>
</template>

<style scoped>
[data-thumbnail-capturing] :deep([data-thumbnail-exclude]) {
  visibility: hidden;
}
</style>
