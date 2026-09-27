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
  sceneNode: CanvasSceneNode;
  palette: CanvasPalette;
  target: CanvasConnectorTarget;
  zoom: number;
}>();
</script>

<template>
  <KonvaGroup :config="{ listening: false }">
    <KonvaRect
      :config="{
        x: sceneNode.layout.frame.x,
        y: sceneNode.layout.frame.y,
        width: sceneNode.layout.frame.width,
        height: sceneNode.layout.frame.height,
        rotation: sceneNode.layout.frame.rotation,
        stroke: palette.accent,
        strokeWidth: 2 / zoom,
        cornerRadius: 10,
      }"
    />
    <KonvaCircle
      :config="{
        ...target.anchor,
        radius: 6 / zoom,
        fill: palette.accent,
        stroke: palette.panel,
        strokeWidth: 2 / zoom,
      }"
    />
  </KonvaGroup>
</template>
