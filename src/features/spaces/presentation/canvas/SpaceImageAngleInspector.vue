<script setup lang="ts">
import { nextTick, onMounted, toRef, useTemplateRef } from "vue";

import { imageEditOf } from "../../image-editing/image-edit";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import SpaceImageAnglePanel from "./SpaceImageAnglePanel.vue";
import { useImageGenerateActions } from "./use-image-generate-actions";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const emit = defineEmits<{ close: [] }>();
const panel = useTemplateRef<HTMLElement>("panel");
const { busy, availability, help, openAiSettings, run } = useImageGenerateActions(
  toRef(props, "node"),
  "angle",
  toRef(() => true),
  () => emit("close"),
);

onMounted(async () => {
  await nextTick();
  panel.value?.querySelector<HTMLElement>("button:not(:disabled)")?.focus({ preventScroll: true });
});
</script>

<template>
  <div
    ref="panel"
    class="w-max max-w-[calc(100vw-2rem)] rounded-panel border border-stroke-strong bg-surface-overlay p-3 shadow-popover"
  >
    <SpaceImageAnglePanel
      :path="node.data.assetPath"
      :revision="node.data.sourceRevision"
      :edit="imageEditOf(node.data)"
      :busy="busy"
      :availability="availability"
      :help="help"
      @cancel="emit('close')"
      @open-settings="openAiSettings"
      @apply="run"
    />
  </div>
</template>
