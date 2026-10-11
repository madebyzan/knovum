<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Label as KonvaLabel, Tag as KonvaTag, Text as KonvaText } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Label";

import type { SectionDropPreview } from "../../../structure/section-drop-reparent";
import type { CanvasPalette } from "./use-canvas-palette";

defineProps<{ action: SectionDropPreview["kind"]; palette: CanvasPalette; zoom: number }>();
const { t } = useI18n();
</script>

<template>
  <KonvaLabel :config="{ x: 12 / zoom, y: -(palette.typography.captionSize + 20) / zoom, listening: false }">
    <KonvaTag
      :config="{
        fill: palette.panel,
        stroke: palette.accent,
        strokeWidth: 1 / zoom,
        cornerRadius: 4 / zoom,
      }"
    />
    <KonvaText
      :config="{
        text:
          action === 'enter'
            ? t('spaces.canvas.sections.drop.enter')
            : t('spaces.canvas.sections.drop.leave'),
        fill: palette.foreground,
        fontFamily: palette.typography.fontFamily,
        fontSize: palette.typography.captionSize / zoom,
        padding: 6 / zoom,
      }"
    />
  </KonvaLabel>
</template>
