<script setup lang="ts">
import { Stage as KonvaStage } from "vue-konva/core";

import type { CanvasDocumentV1 } from "../contract/canvas-document";
import type { CanvasNodeContentSlotProps, CanvasSurfaceProps } from "../contract/canvas-surface";
import CanvasDomOverlay from "./presentation/CanvasDomOverlay.vue";
import CanvasGridLayer from "./presentation/CanvasGridLayer.vue";
import CanvasMarquee from "./presentation/CanvasMarquee.vue";
import CanvasSceneLayers from "./presentation/CanvasSceneLayers.vue";
import { useKonvaCanvasRuntime } from "./use-konva-canvas-runtime";

const props = defineProps<CanvasSurfaceProps>();
const emit = defineEmits<{ documentChange: [document: CanvasDocumentV1] }>();
defineSlots<{ "node-content"(props: CanvasNodeContentSlotProps): unknown }>();
const runtime = useKonvaCanvasRuntime(props, (document) => emit("documentChange", document));
defineExpose({ flushViewport: runtime.viewport.flushViewport });
</script>

<template>
  <div
    ref="host"
    data-space-thumbnail-source
    class="relative h-full min-h-0 w-full overflow-hidden outline-none"
    :class="[
      runtime.connectors.draft.value && 'cursor-crosshair',
      runtime.viewport.panning.value
        ? 'cursor-grabbing'
        : runtime.viewport.spacePressed.value || tool === 'pan'
          ? 'cursor-grab'
          : tool === 'note' || tool === 'text'
            ? 'cursor-crosshair'
            : '',
    ]"
    tabindex="0"
    @keydown="interactionEnabled && runtime.viewport.key($event, true)"
  >
    <KonvaStage
      ref="stage"
      :config="{
        width: runtime.viewport.size.width,
        height: runtime.viewport.size.height,
        x: runtime.viewport.viewport.value.x,
        y: runtime.viewport.viewport.value.y,
        scaleX: runtime.viewport.viewport.value.zoom,
        scaleY: runtime.viewport.viewport.value.zoom,
      }"
      @pointerdown="interactionEnabled && runtime.pointer.pointerDown($event)"
      @pointermove="interactionEnabled && runtime.connectors.pointerMove($event)"
      @pointercancel="runtime.pointer.pointerCancel($event.evt)"
      @pointerup="interactionEnabled && runtime.connectors.pointerUp($event)"
      @wheel="interactionEnabled && runtime.viewport.wheel($event)"
    >
      <CanvasGridLayer
        :palette="runtime.palette.value"
        :size="runtime.viewport.size"
        :viewport="runtime.viewport.viewport.value"
      />
      <CanvasSceneLayers
        :interactive="
          interactionEnabled &&
          tool === 'select' &&
          !runtime.viewport.panning.value &&
          !runtime.capturesCanvasInteraction.value &&
          !runtime.viewport.spacePressed.value
        "
        :selection-chrome-visible="
          interactionEnabled &&
          tool === 'select' &&
          !runtime.viewport.spacePressed.value &&
          !runtime.viewport.panning.value &&
          runtime.gestures.directManipulation.value !== 'drag'
        "
        :connector-draft="runtime.connectors.draft.value"
        :feedback="runtime.gestures.feedback.value"
        :palette="runtime.palette.value"
        :layout-previews="runtime.scenePreview.layouts"
        :relation-previews="runtime.scenePreview.relations"
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
        @drag-move="runtime.nodeDrag.move"
        @drag-start="runtime.nodeDrag.begin"
        @pointer-down="runtime.pointer.nodePointerDown"
        @relation-pointer-down="runtime.selection.selectRelation"
        @resize-snap="runtime.gestures.controller.updateResizeGuides"
        @transform="runtime.gestures.controller.previewTransform($event)"
        @transform-end="runtime.finishTransform"
        @transform-start="runtime.gestures.beginDirectManipulation('transform')"
      />
      <CanvasMarquee
        :bounds="runtime.selection.marqueeRect.value"
        :palette="runtime.palette.value"
        :zoom="runtime.viewport.viewport.value.zoom"
      />
    </KonvaStage>
    <Teleport v-if="runtime.stage.value" :to="runtime.stage.value.getContent()">
      <CanvasDomOverlay
        :interaction-enabled="
          interactionEnabled &&
          tool === 'select' &&
          !runtime.viewport.spacePressed.value &&
          !runtime.viewport.panning.value
        "
        :interactive-node-id="interactionEnabled ? runtime.interactiveNodeId.value : undefined"
        :layout-previews="runtime.scenePreview.layouts"
        :scene="runtime.scene.value"
        :viewport="runtime.viewport.viewport.value"
        :visible-node-ids="runtime.visibility.visibleNodeIds.value"
        :selected-node-ids="runtime.selection.nodeIds.value"
      >
        <template #default="content">
          <slot name="node-content" v-bind="content" />
        </template>
      </CanvasDomOverlay>
    </Teleport>
  </div>
</template>

<style scoped>
[data-thumbnail-capturing] :deep([data-thumbnail-exclude]) {
  visibility: hidden;
}
</style>
