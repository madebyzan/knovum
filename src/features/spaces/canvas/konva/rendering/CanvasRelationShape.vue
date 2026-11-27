<script setup lang="ts">
import type Konva from "konva";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  Arrow as KonvaArrow,
  Group as KonvaGroup,
  Rect as KonvaRect,
  Text as KonvaText,
} from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Arrow";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Text";

import type { CanvasPalette } from "../../use-canvas-palette";
import type { CanvasSceneRelation } from "../scene/canvas-scene";
import {
  canvasRelationArrowheads,
  connectorLabelPosition,
  measureRelationLabelWidth,
} from "./connector-presentation";

const LABEL_FONT_SIZE = 11;
const LABEL_PADDING_X = 6;
const LABEL_HEIGHT = 18;

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
const labelWidth = computed(
  () =>
    measureRelationLabelWidth(label.value, props.palette.typography.fontFamily, LABEL_FONT_SIZE) +
    LABEL_PADDING_X * 2,
);

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
      strokeWidth: selected ? 2 / zoom : 1.5 / zoom,
      pointerLength: 7 / zoom,
      pointerWidth: 6 / zoom,
      lineCap: 'round',
      lineJoin: 'round',
      pointerAtBeginning: canvasRelationArrowheads(relation).source,
      pointerAtEnding: canvasRelationArrowheads(relation).target,
      hitStrokeWidth: 12 / zoom,
      onPointerdown: select,
    }"
  />
  <KonvaGroup
    :config="{
      ...connectorLabelPosition(relation),
      listening: false,
      perfectDrawEnabled: false,
    }"
  >
    <KonvaRect
      :config="{
        x: -labelWidth / 2 / zoom,
        y: -LABEL_HEIGHT / 2 / zoom,
        width: labelWidth / zoom,
        height: LABEL_HEIGHT / zoom,
        fill: palette.panel,
        stroke: selected ? palette.relationByColor[relation.color] : palette.border,
        strokeWidth: (selected ? 1.25 : 0.75) / zoom,
        cornerRadius: 4 / zoom,
      }"
    />
    <KonvaText
      :config="{
        x: -labelWidth / 2 / zoom,
        y: -LABEL_HEIGHT / 2 / zoom,
        width: labelWidth / zoom,
        height: LABEL_HEIGHT / zoom,
        text: label,
        align: 'center',
        verticalAlign: 'middle',
        fontSize: LABEL_FONT_SIZE / zoom,
        fontFamily: palette.typography.fontFamily,
        fill: palette.relationByColor[relation.color],
      }"
    />
  </KonvaGroup>
</template>
