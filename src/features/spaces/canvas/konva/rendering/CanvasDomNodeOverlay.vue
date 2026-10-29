<script setup lang="ts">
import { computed } from "vue";

import type { CanvasPreviewPolicy } from "../../preview/policy";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasSceneNode } from "../scene/canvas-scene";
import CanvasNodeBadges from "./CanvasNodeBadges.vue";

const props = defineProps<{
  previewPolicy: CanvasPreviewPolicy;
  interactive: boolean;
  sceneNode: CanvasSceneNode;
  layoutPreviewByNodeId: ReadonlyMap<string, CanvasNodeLayoutV1>;
  selected: boolean;
  /** Outputs derived from this node; selecting it draws their lineage instead. */
  derivedOutputs: number;
  /** Sources changed since this node was kept. */
  sourceUpdates: number;
}>();
const effectiveLayout = computed(
  () => props.layoutPreviewByNodeId.get(props.sceneNode.node.id) ?? props.sceneNode.layout,
);
const effectiveFrame = computed(() => effectiveLayout.value.frame);
const style = computed(() => ({
  left: `${effectiveFrame.value.x}px`,
  top: `${effectiveFrame.value.y}px`,
  width: `${effectiveFrame.value.width}px`,
  height: `${effectiveFrame.value.height}px`,
  zIndex: props.sceneNode.paintOrder,
  transform: `rotate(${effectiveFrame.value.rotation}deg)`,
  transformOrigin: "top left",
}));
</script>

<template>
  <div
    class="pointer-events-none absolute overflow-visible"
    :data-canvas-node-id="sceneNode.node.id"
    :style="style"
  >
    <slot
      :layout="effectiveLayout"
      :node="sceneNode.node"
      :preview-policy="previewPolicy"
      :interactive="interactive"
      :selected="selected"
    />
    <CanvasNodeBadges
      v-if="!interactive"
      :node-width="effectiveFrame.width"
      :source-updates="sourceUpdates"
      :derived-outputs="selected ? 0 : derivedOutputs"
    />
  </div>
</template>
