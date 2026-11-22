<script setup lang="ts">
import type Konva from "konva";
import { useTemplateRef, watch } from "vue";
import { Arrow as KonvaArrow, type VueKonvaRef } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Arrow";

import type { CanvasPalette } from "../../use-canvas-palette";
import type { CanvasConnectorDraft } from "../interaction/connector-gesture-controller";
import type { CanvasSceneRelation } from "../scene/canvas-scene";
import { draftRelationPath } from "../scene/relation-path";
import { KonvaCanvasLayer } from "./canvas-render-layer";
import CanvasRelationShape from "./CanvasRelationShape.vue";

defineProps<{
  interactive: boolean;
  connectorDraft: CanvasConnectorDraft | undefined;
  palette: CanvasPalette;
  relations: readonly CanvasSceneRelation[];
  relationPreviewById: ReadonlyMap<string, CanvasSceneRelation>;
  selectedRelationIds: ReadonlySet<string>;
  zoom: number;
}>();
const emit = defineEmits<{ pointerDown: [relationIds: readonly string[], event: PointerEvent] }>();
const relationsLayer = useTemplateRef<VueKonvaRef<Konva.Layer>>("relationsLayer");

function syncRelationsCanvas() {
  const canvas = relationsLayer.value?.getNode().getNativeCanvasElement();
  if (!canvas) return;
  // Sits above DOM node cards (z-17) so curves are never occluded, while staying below
  // world overlays (sibling z-18) and Konva selection/connector handles (z-20).
  canvas.style.zIndex = "18";
  canvas.style.pointerEvents = "none";
}

watch(relationsLayer, syncRelationsCanvas, { flush: "post" });
</script>

<template>
  <KonvaCanvasLayer
    v-if="relations.length || connectorDraft"
    ref="relationsLayer"
    :config="{ id: 'canvas-relations-layer', perfectDrawEnabled: false, listening: interactive }"
  >
    <CanvasRelationShape
      v-for="relation in relations"
      :key="relation.key"
      :source-relation="relation"
      :palette="palette"
      :relation-preview-by-id="relationPreviewById"
      :selected="selectedRelationIds.has(relation.key)"
      :zoom="zoom"
      @pointer-down="(relationIds, event) => emit('pointerDown', relationIds, event)"
    />
    <KonvaArrow
      v-if="connectorDraft"
      :config="{
        points: draftRelationPath({
          from: connectorDraft.from,
          fromSide: connectorDraft.sourceSide,
          to: connectorDraft.to,
          toSide: connectorDraft.target?.side,
        }),
        bezier: true,
        stroke: palette.accent,
        fill: palette.accent,
        strokeWidth: 1.5 / zoom,
        pointerLength: 7 / zoom,
        pointerWidth: 6 / zoom,
        lineCap: 'round',
        lineJoin: 'round',
        dash: connectorDraft.target ? undefined : [5 / zoom, 4 / zoom],
        listening: false,
      }"
    />
  </KonvaCanvasLayer>
</template>
