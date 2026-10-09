<script setup lang="ts">
import { computed, provide } from "vue";

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import { canvasPreviewPolicy } from "../../preview/policy";
import { canvasPreviewZoomKey } from "../../preview/viewport-zoom";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasScene } from "../scene/canvas-scene";
import CanvasDomNodeOverlay from "./CanvasDomNodeOverlay.vue";
import { sceneNodesInPaintOrder } from "./use-canvas-scene-layer-projection";

const props = defineProps<{
  scene: CanvasScene;
  interactionEnabled: boolean;
  renderingEnabled: boolean;
  layoutPreviewByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>;
  interactiveNodeId: string | undefined;
  viewport: CanvasViewport;
  visibleNodeIds: ReadonlySet<string>;
  selectedNodeIds: ReadonlySet<string>;
}>();
provide(
  canvasPreviewZoomKey,
  computed(() => props.viewport.zoom),
);
const previewPolicy = computed(() =>
  canvasPreviewPolicy(props.viewport.zoom * 100, props.scene.nodes.length),
);
const visibleNodes = computed(() =>
  sceneNodesInPaintOrder(
    props.scene,
    props.renderingEnabled
      ? props.visibleNodeIds
      : new Set(props.interactiveNodeId ? [props.interactiveNodeId] : []),
  ),
);
const worldStyle = computed(() => ({
  transform: `translate(${props.viewport.x}px, ${props.viewport.y}px) scale(${props.viewport.zoom})`,
}));
</script>

<template>
  <div
    :inert="!interactionEnabled || !renderingEnabled"
    class="pointer-events-none absolute inset-0 z-[17] overflow-hidden select-none"
  >
    <div class="absolute top-0 left-0 origin-top-left" :style="worldStyle">
      <CanvasDomNodeOverlay
        v-for="sceneNode in visibleNodes"
        :key="sceneNode.node.id"
        :preview-policy="previewPolicy"
        :interactive="interactiveNodeId === sceneNode.node.id"
        :scene-node="sceneNode"
        :layout-preview-by-node-id="layoutPreviewByNodeId"
        :selected="selectedNodeIds.has(sceneNode.node.id)"
      >
        <template #default="content">
          <slot v-bind="content" />
        </template>
      </CanvasDomNodeOverlay>
    </div>
  </div>
</template>
