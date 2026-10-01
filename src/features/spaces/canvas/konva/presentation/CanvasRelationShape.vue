<script setup lang="ts">
import type Konva from "konva";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Arrow as KonvaArrow, Text as KonvaText } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Arrow";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Text";

import type { CanvasSceneRelation } from "../scene/canvas-scene";
import { canvasRelationArrowheads, connectorLabelPosition } from "./connector-presentation";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  sourceRelation: CanvasSceneRelation;
  palette: CanvasPalette;
  previews: ReadonlyMap<string, CanvasSceneRelation>;
  selected: boolean;
  zoom: number;
}>();
const emit = defineEmits<{ pointerDown: [id: string, event: PointerEvent] }>();
const { t } = useI18n();
const relation = computed(() => props.previews.get(props.sourceRelation.id) ?? props.sourceRelation);

function select(event: Konva.KonvaEventObject<PointerEvent>) {
  event.cancelBubble = true;
  emit("pointerDown", relation.value.id, event.evt);
}
</script>

<template>
  <KonvaArrow
    :config="{
      points: relation.points,
      bezier: true,
      stroke: palette.relation[relation.color],
      fill: palette.relation[relation.color],
      strokeWidth: selected ? 3 / zoom : 2.5 / zoom,
      pointerLength: 9 / zoom,
      pointerWidth: 8 / zoom,
      lineCap: 'round',
      lineJoin: 'round',
      pointerAtBeginning: canvasRelationArrowheads(relation).source,
      pointerAtEnding: canvasRelationArrowheads(relation).target,
      hitStrokeWidth: 14 / zoom,
    }"
    @pointerdown="select"
  />
  <KonvaText
    :config="{
      ...connectorLabelPosition(relation),
      text: t(`spaces.knowledge.relations.kinds.${relation.label}`),
      offsetX: 80 / zoom,
      offsetY: 8 / zoom,
      width: 160 / zoom,
      align: 'center',
      fontSize: 12 / zoom,
      fill: palette.relation[relation.color],
      stroke: palette.panel,
      strokeWidth: 5 / zoom,
      fillAfterStrokeEnabled: true,
      listening: false,
    }"
  />
</template>
