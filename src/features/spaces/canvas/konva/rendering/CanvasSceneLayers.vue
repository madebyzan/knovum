<script setup lang="ts">
import type Konva from "konva";
import { computed, shallowRef, toRef, useTemplateRef, watch } from "vue";
import type { VueKonvaRef } from "vue-konva/core";

import type { CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasGestureFeedback, CanvasSnapGuide } from "../interaction/canvas-snapping";
import type { CanvasConnectorDraft } from "../interaction/connector-gesture-controller";
import type { CanvasScene, CanvasSceneRelation } from "../scene/canvas-scene";
import type { CanvasConnectorSide } from "../scene/connector-geometry";
import type { CanvasSelectionTransformPolicy } from "../selection/selection-transform-policy";
import { KonvaCanvasLayer } from "./canvas-render-layer";
import CanvasConnectionTargetHighlight from "./CanvasConnectionTargetHighlight.vue";
import CanvasConnectorControls from "./CanvasConnectorControls.vue";
import CanvasInteractionFeedback from "./CanvasInteractionFeedback.vue";
import CanvasNodeShape from "./CanvasNodeShape.vue";
import CanvasRelationsLayer from "./CanvasRelationsLayer.vue";
import CanvasSelectionTransformer from "./CanvasSelectionTransformer.vue";
import { useCanvasOverviewLayer } from "./use-canvas-overview-layer";
import type { CanvasPalette } from "./use-canvas-palette";
import { sceneNodesInPaintOrder, useCanvasSceneLayerProjection } from "./use-canvas-scene-layer-projection";

function relayPointerDown(nodeId: string, event: Konva.KonvaEventObject<PointerEvent>) {
  emit("pointerDown", nodeId, event);
}
const props = defineProps<{
  interactive: boolean;
  selectionChromeVisible: boolean;
  connectorDraft: CanvasConnectorDraft | undefined;
  feedback: CanvasGestureFeedback | undefined;
  palette: CanvasPalette;
  layoutPreviewByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>;
  relationPreviewById: ReadonlyMap<string, CanvasSceneRelation>;
  scene: CanvasScene;
  selectedNodeIds: ReadonlySet<string>;
  selectedRelationIds: ReadonlySet<string>;
  transformPolicy: CanvasSelectionTransformPolicy;
  zoom: number;
  stage?: Konva.Stage;
  visibleNodeIds: ReadonlySet<string>;
  /** The visible nodes that mount their content; every other visible node is drawn as its overview. */
  detailNodeIds: ReadonlySet<string>;
  visibleRelationIds: ReadonlySet<string>;
}>();
const emit = defineEmits<{
  activate: [nodeId: string];
  connectorStart: [nodeId: string, side: CanvasConnectorSide];
  dragEnd: [nodeId: string];
  dragMove: [nodeId: string, event: Konva.KonvaEventObject<DragEvent>];
  dragStart: [nodeId: string, event: Konva.KonvaEventObject<DragEvent>];
  pointerDown: [nodeId: string, event: Konva.KonvaEventObject<PointerEvent>];
  relationPointerDown: [relationIds: readonly string[], event: PointerEvent];
  resizeSnap: [guides: readonly CanvasSnapGuide[]];
  transform: [proportionalAnchor?: string];
  transformEnd: [];
  transformStart: [];
}>();
const transformingSelection = shallowRef(false);
const interactionLayer = useTemplateRef<VueKonvaRef<Konva.Layer>>("interactionLayer");
const overviewLayer = useTemplateRef<VueKonvaRef<Konva.Layer>>("overviewLayer");
useCanvasOverviewLayer({
  layer: computed(() => overviewLayer.value?.getNode()),
  nodes: computed(() =>
    sceneNodesInPaintOrder(
      props.scene,
      new Set([...props.visibleNodeIds].filter((id) => !props.detailNodeIds.has(id))),
    ),
  ),
  layoutPreviewByNodeId: () => props.layoutPreviewByNodeId,
  palette: toRef(props, "palette"),
  zoom: toRef(props, "zoom"),
});
const {
  connectorNode,
  connectorTargetNode,
  resizeSnapFrames,
  selectedFrames,
  visibleContentNodes,
  visibleRelations,
} = useCanvasSceneLayerProjection(props);

function beginSelectionTransform() {
  transformingSelection.value = true;
  emit("transformStart");
}

function finishSelectionTransform() {
  transformingSelection.value = false;
  emit("transformEnd");
}

function syncInteractionCanvas() {
  const canvas = interactionLayer.value?.getNode().getNativeCanvasElement();
  if (!canvas) return;
  canvas.style.zIndex = "20";
  canvas.style.pointerEvents = "none";
  canvas.dataset.thumbnailExclude = "";
}

watch(interactionLayer, syncInteractionCanvas, { flush: "post" });
</script>

<template>
  <CanvasRelationsLayer
    :interactive="interactive"
    :connector-draft="connectorDraft"
    :palette="palette"
    :relations="visibleRelations"
    :relation-preview-by-id="relationPreviewById"
    :selected-relation-ids="selectedRelationIds"
    :zoom="zoom"
    @pointer-down="(relationIds, event) => emit('relationPointerDown', relationIds, event)"
  />
  <KonvaCanvasLayer ref="overviewLayer" :config="{ id: 'canvas-overview-layer', listening: false }" />
  <KonvaCanvasLayer :config="{ id: 'canvas-node-layer', listening: interactive }">
    <CanvasNodeShape
      :interactive="interactive"
      v-for="sceneNode in visibleContentNodes"
      :key="sceneNode.node.id"
      :scene-node="sceneNode"
      :palette="palette"
      :zoom="zoom"
      @activate="emit('activate', $event)"
      @drag-end="emit('dragEnd', $event)"
      @drag-move="(nodeId, event) => emit('dragMove', nodeId, event)"
      @drag-start="(id, event) => emit('dragStart', id, event)"
      @pointer-down="relayPointerDown"
    />
  </KonvaCanvasLayer>
  <KonvaCanvasLayer :config="{ id: 'canvas-drag-layer', listening: false }" />
  <KonvaCanvasLayer v-if="selectedFrames.length || connectorDraft || feedback" ref="interactionLayer">
    <CanvasSelectionTransformer
      v-if="stage && !connectorDraft"
      :interactive="interactive"
      :visible="selectionChromeVisible"
      :policy="transformPolicy"
      :palette="palette"
      :selected-frames="selectedFrames"
      :snap-frames="resizeSnapFrames"
      :zoom="zoom"
      :stage="stage"
      @transform="emit('transform', $event)"
      @transform-start="beginSelectionTransform"
      @transform-end="finishSelectionTransform"
      @resize-snap="emit('resizeSnap', $event)"
    />
    <CanvasConnectorControls
      :interactive="interactive"
      :node="connectorNode"
      :palette="palette"
      :visible="selectionChromeVisible && !connectorDraft && !transformingSelection"
      :zoom="zoom"
      @start="(nodeId, side) => emit('connectorStart', nodeId, side)"
    />
    <CanvasConnectionTargetHighlight
      v-if="connectorDraft?.target && connectorTargetNode"
      :scene-node="connectorTargetNode"
      :palette="palette"
      :target="connectorDraft.target"
      :zoom="zoom"
    />
    <CanvasInteractionFeedback :feedback="feedback" :palette="palette" :zoom="zoom" />
  </KonvaCanvasLayer>
</template>
