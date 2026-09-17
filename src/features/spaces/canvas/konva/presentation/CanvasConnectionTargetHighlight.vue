<script setup lang="ts">
import { Circle as KonvaCircle, Group as KonvaGroup, Rect as KonvaRect } from "vue-konva/core";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Circle";
// oxlint-disable-next-line import/no-unassigned-import
import "konva/lib/shapes/Rect";

import type { CanvasConnectorTarget } from "../interaction/connector-gesture-controller";
import type { CanvasSceneNode } from "../scene/scene-projection";
import type { CanvasPalette } from "./use-canvas-palette";

defineProps<{
  item: CanvasSceneNode;
  palette: CanvasPalette;
  target: CanvasConnectorTarget;
  zoom: number;
}>();
</script>

<template>
  <KonvaGroup :config="{ listening: false }">
    <KonvaRect
      :config="{
        x: item.layout.frame.x,
        y: item.layout.frame.y,
        width: item.layout.frame.width,
        height: item.layout.frame.height,
        rotation: item.layout.frame.rotation,
        stroke: palette.accent,
        strokeWidth: 2 / zoom,
        cornerRadius: item.node.data.kind === 'section' ? 8 : 10,
      }"
    />
    <KonvaCircle
      :config="{
        ...target.anchor,
        radius: 6 / zoom,
        fill: palette.accent,
        stroke: palette.surface,
        strokeWidth: 2 / zoom,
      }"
    />
  </KonvaGroup>
</template>
