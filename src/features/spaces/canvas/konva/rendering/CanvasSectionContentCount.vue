<script setup lang="ts">
import { Group as KonvaGroup, Rect as KonvaRect, Text as KonvaText } from "vue-konva/core";

import type { CanvasPalette } from "./use-canvas-palette";

defineProps<{ x: number; width: number; text: string; palette: CanvasPalette }>();
</script>

<template>
  <KonvaGroup :config="{ x, y: 11, listening: false }">
    <KonvaRect :config="{ width, height: 18, fill: palette.border, cornerRadius: 9, opacity: 0.8 }" />
    <KonvaText
      :config="{
        width,
        height: 18,
        text,
        align: 'center',
        verticalAlign: 'middle',
        fill: palette.foregroundSecondary,
        fontFamily: palette.typography.fontFamily,
        fontSize: Math.max(10, palette.typography.captionSize - 1),
        fontStyle: '500',
      }"
    />
  </KonvaGroup>
</template>
