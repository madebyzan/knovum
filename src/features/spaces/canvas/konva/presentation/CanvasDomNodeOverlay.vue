<script setup lang="ts">
import { computed } from "vue";

import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasSceneContentNode } from "../scene/scene-projection";

const props = defineProps<{
  detailLevel: "compact" | "summary" | "full";
  interactive: boolean;
  item: CanvasSceneContentNode;
  layoutPreviews: ReadonlyMap<string, CanvasNodeLayoutV1>;
  selected: boolean;
}>();
const effectiveLayout = computed(() => props.layoutPreviews.get(props.item.node.id) ?? props.item.layout);
const contentScale = computed(() => effectiveLayout.value.contentScale ?? 1);
const contentLayout = computed(() => ({
  ...effectiveLayout.value,
  frame: {
    ...effectiveLayout.value.frame,
    width: effectiveLayout.value.frame.width / contentScale.value,
    height: effectiveLayout.value.frame.height / contentScale.value,
  },
}));
const effectiveFrame = computed(() => effectiveLayout.value.frame);
const style = computed(() => ({
  left: `${effectiveFrame.value.x}px`,
  top: `${effectiveFrame.value.y}px`,
  width: `${effectiveFrame.value.width / contentScale.value}px`,
  height: `${effectiveFrame.value.height / contentScale.value}px`,
  zIndex: props.item.paintOrder,
  transform: `rotate(${effectiveFrame.value.rotation}deg) scale(${contentScale.value})`,
  transformOrigin: "top left",
}));
</script>

<template>
  <div class="pointer-events-none absolute overflow-visible" :style="style">
    <slot
      :layout="contentLayout"
      :node="item.node"
      :detail-level="detailLevel"
      :interactive="interactive"
      :selected="selected"
    />
  </div>
</template>
