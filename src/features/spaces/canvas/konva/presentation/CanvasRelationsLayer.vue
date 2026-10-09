<script setup lang="ts">
import { Arrow as KonvaArrow, Layer as KonvaLayer } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Arrow";

import type { CanvasConnectorDraft } from "../interaction/connector-gesture-controller";
import type { CanvasSceneRelation } from "../scene/canvas-scene";
import { draftRelationPath } from "../scene/relation-path";
import CanvasRelationShape from "./CanvasRelationShape.vue";
import type { CanvasPalette } from "./use-canvas-palette";

defineProps<{
  interactive: boolean;
  connectorDraft: CanvasConnectorDraft | undefined;
  palette: CanvasPalette;
  relations: readonly CanvasSceneRelation[];
  relationPreviews: ReadonlyMap<string, CanvasSceneRelation>;
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
      :previews="relationPreviews"
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
