<script setup lang="ts">
import { Arrow as KonvaArrow } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Arrow";

import type { CanvasConnectorDraft } from "../interaction/connector-gesture-controller";
import type { CanvasSceneRelation } from "../scene/canvas-scene";
import { draftRelationPath } from "../scene/relation-path";
import { useCanvasRenderLayer } from "./canvas-render-layer";
import CanvasRelationShape from "./CanvasRelationShape.vue";
import type { CanvasPalette } from "./use-canvas-palette";

const KonvaLayer = useCanvasRenderLayer();
defineProps<{
  interactive: boolean;
  connectorDraft: CanvasConnectorDraft | undefined;
  palette: CanvasPalette;
  relations: readonly CanvasSceneRelation[];
  relationPreviewById: ReadonlyMap<string, CanvasSceneRelation>;
  selectedRelationIds: ReadonlySet<string>;
  zoom: number;
}>();
const emit = defineEmits<{ pointerDown: [id: string, event: PointerEvent] }>();
</script>

<template>
  <KonvaLayer
    v-if="relations.length || connectorDraft"
    :config="{ perfectDrawEnabled: false, listening: interactive }"
  >
    <CanvasRelationShape
      v-for="relation in relations"
      :key="relation.key"
      :source-relation="relation"
      :palette="palette"
      :relation-preview-by-id="relationPreviewById"
      :selected="selectedRelationIds.has(relation.key)"
      :zoom="zoom"
      @pointer-down="(id, event) => emit('pointerDown', id, event)"
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
        strokeWidth: 2.25 / zoom,
        pointerLength: 9 / zoom,
        pointerWidth: 8 / zoom,
        lineCap: 'round',
        lineJoin: 'round',
        dash: connectorDraft.target ? undefined : [6 / zoom, 4 / zoom],
        listening: false,
      }"
    />
  </KonvaLayer>
</template>
