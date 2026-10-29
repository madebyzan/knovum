<script setup lang="ts">
import { computed, provide, useTemplateRef } from "vue";

import { canvasPreviewPolicy } from "../../preview/policy";
import { canvasPreviewZoomKey } from "../../preview/viewport-zoom";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasScene } from "../scene/canvas-scene";
import type { CanvasCamera } from "../viewport/canvas-camera";
import { useCameraStyle, worldTransform } from "../viewport/use-camera-style";
import CanvasDomNodeOverlay from "./CanvasDomNodeOverlay.vue";
import { sceneNodesInPaintOrder } from "./use-canvas-scene-layer-projection";

const props = defineProps<{
  camera: CanvasCamera;
  scene: CanvasScene;
  interactionEnabled: boolean;
  renderingEnabled: boolean;
  layoutPreviewByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>;
  interactiveNodeId: string | undefined;
  visibleNodeIds: ReadonlySet<string>;
  selectedNodeIds: ReadonlySet<string>;
  derivedOutputCountByNodeId: ReadonlyMap<string, number>;
  sourceUpdateCountByNodeId: ReadonlyMap<string, number>;
  zoom: number;
}>();
const world = useTemplateRef<HTMLElement>("world");
provide(
  canvasPreviewZoomKey,
  computed(() => props.zoom),
);
const previewPolicy = computed(() => canvasPreviewPolicy(props.zoom * 100, props.scene.nodes.length));
const visibleNodes = computed(() => sceneNodesInPaintOrder(props.scene, props.visibleNodeIds));
useCameraStyle(props.camera, world, worldTransform);
</script>

<template>
  <div
    :inert="!interactionEnabled || !renderingEnabled"
    class="pointer-events-none absolute inset-0 z-[17] overflow-hidden select-none"
  >
    <!-- The world transform is written from the camera frame, so panning never re-renders a node card. -->
    <div ref="world" class="absolute top-0 left-0 origin-top-left will-change-transform">
      <CanvasDomNodeOverlay
        v-for="sceneNode in visibleNodes"
        :key="sceneNode.node.id"
        :preview-policy="previewPolicy"
        :interactive="interactiveNodeId === sceneNode.node.id"
        :scene-node="sceneNode"
        :layout-preview-by-node-id="layoutPreviewByNodeId"
        :selected="selectedNodeIds.has(sceneNode.node.id)"
        :derived-outputs="derivedOutputCountByNodeId.get(sceneNode.node.id) ?? 0"
        :source-updates="sourceUpdateCountByNodeId.get(sceneNode.node.id) ?? 0"
      >
        <template #default="content">
          <slot v-bind="content" />
        </template>
      </CanvasDomNodeOverlay>
    </div>
  </div>
</template>
