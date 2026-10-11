<script setup lang="ts">
import { computed } from "vue";
import { Stage as KonvaStage } from "vue-konva/core";

import type { CanvasNodeContentSlotProps, CanvasSurfaceProps } from "../contracts/canvas-surface";
import type { CanvasDocumentV1 } from "../schema/document";
import CanvasDomOverlay from "./presentation/CanvasDomOverlay.vue";
import CanvasGridLayer from "./presentation/CanvasGridLayer.vue";
import CanvasMarquee from "./presentation/CanvasMarquee.vue";
import CanvasSceneLayers from "./presentation/CanvasSceneLayers.vue";
import CanvasSectionCandidateFeedback from "./presentation/CanvasSectionCandidateFeedback.vue";
import CanvasStructureHud from "./presentation/CanvasStructureHud.vue";
import CanvasStructureInteractionLayer from "./presentation/CanvasStructureInteractionLayer.vue";
import CanvasStructureSectionsLayer from "./presentation/CanvasStructureSectionsLayer.vue";
import { useKonvaCanvasRuntime } from "./use-konva-canvas-runtime";

const props = defineProps<CanvasSurfaceProps>();
const emit = defineEmits<{ documentChange: [document: CanvasDocumentV1] }>();
defineSlots<{ "node-content"(props: CanvasNodeContentSlotProps): unknown }>();
const runtime = useKonvaCanvasRuntime(props, (document) => emit("documentChange", document));
const selectGesturesEnabled = computed(
  () =>
    props.interactionEnabled &&
    props.canvasGesturesEnabled &&
    props.tool === "select" &&
    !runtime.viewport.panning.value,
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
    class="relative h-full min-h-0 w-full overflow-hidden outline-none"
    :class="cursorClass"
    tabindex="0"
    @contextmenu="interactionEnabled && runtime.contextMenu.pointer($event)"
    @keydown="
      interactionEnabled && !runtime.contextMenu.keyboard($event) && runtime.viewport.key($event, true)
    "
  >
    <KonvaStage
      ref="stage"
      :config="{
        visible: renderingEnabled,
        listening: renderingEnabled,
        width: runtime.viewport.stageSize.width,
        height: runtime.viewport.stageSize.height,
        x: runtime.viewport.viewport.value.x,
        y: runtime.viewport.viewport.value.y,
        scaleX: runtime.viewport.viewport.value.zoom,
        scaleY: runtime.viewport.viewport.value.zoom,
      }"
      @pointerdown="interactionEnabled && canvasGesturesEnabled && runtime.pointer.pointerDown($event)"
      @pointermove="interactionEnabled && canvasGesturesEnabled && runtime.connectors.pointerMove($event)"
      @pointercancel="runtime.pointer.pointerCancel($event.evt)"
      @pointerup="interactionEnabled && canvasGesturesEnabled && runtime.connectors.pointerUp($event)"
      @wheel="interactionEnabled && runtime.viewport.wheel($event)"
    >
      <CanvasGridLayer
        :palette="runtime.palette.value"
        :size="runtime.viewport.stageSize"
        :viewport="runtime.viewport.viewport.value"
      />
      <CanvasStructureSectionsLayer
        :palette="runtime.palette.value"
        :sections="runtime.visibility.visibleSections.value"
        @error="events.onError"
      />
      <CanvasSceneLayers
        :interactive="
          selectGesturesEnabled &&
          !runtime.gestures.busy.value &&
          !runtime.capturesCanvasInteraction.value &&
          !runtime.viewport.spacePressed.value
        "
        :selection-chrome-visible="
          selectGesturesEnabled &&
          !runtime.viewport.spacePressed.value &&
          runtime.directManipulation.value !== 'drag'
        "
        :connector-draft="runtime.connectors.draft.value"
        :feedback="runtime.feedback.value"
        :palette="runtime.palette.value"
        :layout-preview-by-node-id="runtime.scenePreview.layoutByNodeId"
        :relation-preview-by-id="runtime.scenePreview.relationById"
        :scene="runtime.scene.value"
        :selected-node-ids="runtime.selection.nodeIds.value"
        :selected-relation-ids="runtime.visibility.selectedRelationIds.value"
        :transform-policy="runtime.transformPolicy.value"
        :zoom="runtime.viewport.viewport.value.zoom"
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
        :zoom="runtime.viewport.viewport.value.zoom"
      />
      <CanvasStructureInteractionLayer
        :groups="runtime.structure.structureScene.value.groups"
        :group-scope-path="groupScopePath"
        :group-transform-enabled="runtime.canTransformGroups.value"
        :interactive="
          selectGesturesEnabled &&
          !runtime.gestures.busy.value &&
          !runtime.capturesCanvasInteraction.value &&
          !runtime.viewport.spacePressed.value
        "
        :selection-chrome-visible="
          selectGesturesEnabled &&
          !runtime.viewport.spacePressed.value &&
          runtime.directManipulation.value !== 'drag'
        "
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
        @resize-start="
          (id, corner, event) => {
            runtime.pointer.markSelectionDragged();
            runtime.structureGestures.beginResize(
              id,
              corner,
              event.target,
              runtime.pointer.pressedCanvasPoint.value!,
            );
          }
        "
        @resize-move="runtime.structureGestures.resize"
        @pointer-down="runtime.pointer.structurePointerDown"
        @transform-start="
          (id, event) => {
            runtime.pointer.markSelectionDragged();
            runtime.structureGestures.beginTransform(id, event.target);
          }
        "
        @transform="runtime.structureGestures.transform"
      />
      <CanvasMarquee
        :bounds="runtime.selection.marqueeRect.value"
        :palette="runtime.palette.value"
        :zoom="runtime.viewport.viewport.value.zoom"
      />
      <CanvasMarquee
        :bounds="runtime.pointer.sectionDraft.value"
        :palette="runtime.palette.value"
        :zoom="runtime.viewport.viewport.value.zoom"
      />
    </KonvaStage>
    <Teleport v-if="runtime.stage.value" :to="runtime.stage.value.getContent()">
      <CanvasDomOverlay
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
        :viewport="runtime.viewport.viewport.value"
        :visible-node-ids="runtime.visibility.visibleContentNodeIds.value"
        :selected-node-ids="runtime.selection.nodeIds.value"
      >
        <template #default="content">
          <slot name="node-content" v-bind="content" />
        </template>
      </CanvasDomOverlay>
    </Teleport>
    <CanvasStructureHud
      :group-scope-titles="runtime.structure.groupScopeTitles.value"
      :rename-target="runtime.structureInteractions.renameTarget.value"
      :rename-error="runtime.structureInteractions.renameError.value"
      :viewport="runtime.viewport.viewport.value"
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
