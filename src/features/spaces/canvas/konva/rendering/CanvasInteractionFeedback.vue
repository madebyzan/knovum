<script setup lang="ts">
import Konva from "konva";
import { computed, nextTick, onBeforeUnmount, shallowRef, useTemplateRef, watch } from "vue";
import { Group as KonvaGroup, Line as KonvaLine, Rect as KonvaRect, Text as KonvaText } from "vue-konva/core";
import type { VueKonvaRef } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/Tween";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Line";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Text";

import type { CanvasGestureFeedback } from "../interaction/canvas-snapping";
import CanvasSpacingGuide from "./CanvasSpacingGuide.vue";
import type { CanvasPalette } from "./use-canvas-palette";

const props = defineProps<{
  feedback: CanvasGestureFeedback | undefined;
  palette: CanvasPalette;
  zoom: number;
}>();

const displayedResize = shallowRef(resizeFeedback(props.feedback));
const resizeBadgeGroup = useTemplateRef<VueKonvaRef<Konva.Group>>("resizeBadgeGroup");
let resizeFade: Konva.Tween | undefined;
const badge = computed(() => {
  const resize = displayedResize.value?.resize;
  if (!resize) return undefined;
  const text = `W ${Math.round(resize.width)}   H ${Math.round(resize.height)}`;
  const width = Math.max(76, text.length * 7 + 16);
  return { ...resize, text, width };
});

watch(
  () => props.feedback,
  async (feedback) => {
    const resize = resizeFeedback(feedback);
    if (resize) displayedResize.value = resize;
    await nextTick();
    const group = resizeBadgeGroup.value?.getNode();
    if (!group) return;
    resizeFade?.destroy();
    resizeFade = undefined;
    if (resize) {
      group.opacity(1);
      return;
    }
    resizeFade = new Konva.Tween({
      node: group,
      opacity: 0,
      duration: 0.18,
      easing: Konva.Easings.EaseOut,
      onFinish: () => {
        resizeFade?.destroy();
        resizeFade = undefined;
        displayedResize.value = undefined;
      },
    });
    resizeFade.play();
  },
);

onBeforeUnmount(() => resizeFade?.destroy());

function resizeFeedback(feedback: CanvasGestureFeedback | undefined) {
  if (!feedback?.resize) return undefined;
  return {
    resize: feedback.resize,
    measurement: feedback.guides.find(({ kind }) => kind === "measurement"),
  };
}
</script>

<template>
  <KonvaGroup :config="{ listening: false, perfectDrawEnabled: false }">
    <CanvasSpacingGuide
      v-for="(guide, index) in feedback?.guides.filter(({ kind }) => kind === 'spacing') ?? []"
      :key="`spacing:${index}`"
      :guide="guide"
      :palette="palette"
      :zoom="zoom"
    />
    <KonvaLine
      v-for="(guide, index) in feedback?.guides.filter(({ kind }) => kind === 'alignment') ?? []"
      :key="`${guide.kind}:${index}`"
      :config="{
        points: [...guide.points],
        stroke: palette.accent,
        strokeWidth: 1 / zoom,
        dash: [5 / zoom, 4 / zoom],
        lineCap: 'round',
        opacity: 0.72,
      }"
    />
    <KonvaGroup v-if="badge" ref="resizeBadgeGroup">
      <KonvaLine
        v-if="displayedResize?.measurement"
        :config="{
          points: [...displayedResize.measurement.points],
          stroke: palette.measurement,
          strokeWidth: 1.5 / zoom,
          dash: [3 / zoom, 5 / zoom],
          lineCap: 'round',
          opacity: 0.92,
        }"
      />
      <KonvaGroup
        :config="{
          x: badge.x - badge.width / 2 / zoom,
          y: badge.y,
        }"
      >
        <KonvaRect
          :config="{
            width: badge.width / zoom,
            height: 24 / zoom,
            fill: palette.foreground,
            cornerRadius: 6 / zoom,
          }"
        />
        <KonvaText
          :config="{
            x: 8 / zoom,
            y: 5 / zoom,
            width: (badge.width - 16) / zoom,
            text: badge.text,
            align: 'center',
            fontSize: 12 / zoom,
            fill: palette.canvas,
          }"
        />
      </KonvaGroup>
    </KonvaGroup>
  </KonvaGroup>
</template>
