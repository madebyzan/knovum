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
  relationPreviewById: ReadonlyMap<string, CanvasSceneRelation>;
  selected: boolean;
  zoom: number;
}>();
const emit = defineEmits<{ pointerDown: [relationIds: readonly string[], event: PointerEvent] }>();
const { t } = useI18n();
const relation = computed(
  () => props.relationPreviewById.get(props.sourceRelation.id) ?? props.sourceRelation,
);
const relationIds = computed(() => relation.value.relations.map(({ id }) => id));
const label = computed(() => {
  const kind = t(`spaces.knowledge.relations.kinds.${relation.value.label}`);
  const count = relation.value.relations.length;
  return count > 1 ? `${kind} ×${count}` : kind;
});

function select(event: Konva.KonvaEventObject<PointerEvent>) {
  event.cancelBubble = true;
  emit("pointerDown", relationIds.value, event.evt);
}
</script>

<template>
  <KonvaArrow
    :config="{
      points: relation.points,
      relationIds,
      bezier: true,
      stroke: palette.relationByColor[relation.color],
      fill: palette.relationByColor[relation.color],
      strokeWidth: selected ? 3 / zoom : 2.5 / zoom,
      pointerLength: 9 / zoom,
      pointerWidth: 8 / zoom,
      lineCap: 'round',
      lineJoin: 'round',
      pointerAtBeginning: canvasRelationArrowheads(relation).source,
      pointerAtEnding: canvasRelationArrowheads(relation).target,
      hitStrokeWidth: 14 / zoom,
      onPointerdown: select,
    }"
  />
  <KonvaText
    :config="{
      ...connectorLabelPosition(relation),
      text: label,
      offsetX: 80 / zoom,
      offsetY: 8 / zoom,
      width: 160 / zoom,
      align: 'center',
      fontSize: 12 / zoom,
      fill: palette.relationByColor[relation.color],
      stroke: palette.panel,
      strokeWidth: 5 / zoom,
      fillAfterStrokeEnabled: true,
      listening: false,
    }"
  />
</template>
