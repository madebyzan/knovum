<script setup lang="ts">
import { computed } from "vue";

import type { CanvasViewportDetailProfile } from "../../contract/canvas-detail-profile";
import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasSceneNode } from "../scene/scene-projection";

const props = defineProps<{
  detailProfile: CanvasViewportDetailProfile;
  interactive: boolean;
  sceneNode: CanvasSceneNode;
  layoutPreviews: ReadonlyMap<string, CanvasNodeLayoutV1>;
  selected: boolean;
}>();
const effectiveLayout = computed(
  () => props.layoutPreviews.get(props.sceneNode.node.id) ?? props.sceneNode.layout,
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
  <div class="pointer-events-none absolute overflow-visible" :style="style">
    <slot
      :layout="effectiveLayout"
      :node="sceneNode.node"
      :detail-profile="detailProfile"
      :interactive="interactive"
      :selected="selected"
    />
  </div>
</template>
