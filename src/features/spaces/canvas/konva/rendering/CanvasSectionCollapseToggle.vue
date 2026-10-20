<script setup lang="ts">
import { shallowRef } from "vue";
import { Group as KonvaGroup, Line as KonvaLine, Rect as KonvaRect } from "vue-konva/core";

import type { CanvasPalette } from "./use-canvas-palette";

defineProps<{ x: number; collapsed: boolean; palette: CanvasPalette }>();
const emit = defineEmits<{ toggle: [] }>();
const hovered = shallowRef(false);
</script>

<template>
  <KonvaGroup
    :config="{
      x,
      y: 9,
      onClick: () => emit('toggle'),
      onPointerenter: () => (hovered = true),
      onPointerleave: () => (hovered = false),
    }"
  >
    <KonvaRect
      :config="{
        width: 22,
        height: 22,
        fill: hovered ? palette.border : 'rgba(0,0,0,0.001)',
        cornerRadius: 6,
        opacity: hovered ? 0.6 : 1,
      }"
    />
    <KonvaLine
      :config="{
        points: collapsed ? [8, 6, 13, 11, 8, 16] : [6, 8, 11, 13, 16, 8],
        stroke: hovered ? palette.foreground : palette.foregroundSecondary,
        strokeWidth: 1.5,
        lineCap: 'round',
        lineJoin: 'round',
        listening: false,
      }"
    />
  </KonvaGroup>
</template>
