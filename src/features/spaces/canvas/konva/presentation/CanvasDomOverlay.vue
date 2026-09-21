<script setup lang="ts">
import { computed } from "vue";

import { spaceCanvasDetailLevel } from "../../../assets/preview-policy";
import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasViewport } from "../../contract/canvas-editor";
import type { CanvasScene } from "../scene/scene-projection";
import CanvasDomNodeOverlay from "./CanvasDomNodeOverlay.vue";

const props = defineProps<{
  scene: CanvasScene;
  interactionEnabled: boolean;
  layoutPreviews: ReadonlyMap<string, CanvasNodeLayoutV1>;
  interactiveNodeId: string | undefined;
  viewport: CanvasViewport;
  visibleNodeIds: ReadonlySet<string>;
  selectedNodeIds: ReadonlySet<string>;
}>();
const detailLevel = computed(() =>
  spaceCanvasDetailLevel(props.viewport.zoom * 100, props.scene.nodes.length),
);
const visibleNodes = computed(() =>
  props.scene.nodes.filter(({ node }) => props.visibleNodeIds.has(node.id)),
);
const worldStyle = computed(() => ({
  transform: `translate(${props.viewport.x}px, ${props.viewport.y}px) scale(${props.viewport.zoom})`,
}));
</script>

<template>
  <div
    :inert="!interactionEnabled"
    class="pointer-events-none absolute inset-0 z-10 overflow-hidden select-none"
  >
    <div class="absolute top-0 left-0 origin-top-left" :style="worldStyle">
      <CanvasDomNodeOverlay
        v-for="sceneNode in visibleNodes"
        :key="sceneNode.node.id"
        :detail-level="detailLevel"
        :interactive="interactiveNodeId === sceneNode.node.id"
        :scene-node="sceneNode"
        :layout-previews="layoutPreviews"
        :selected="selectedNodeIds.has(sceneNode.node.id)"
      >
        <template #default="content">
          <slot v-bind="content" />
        </template>
      </CanvasDomNodeOverlay>
    </div>
  </div>
</template>
