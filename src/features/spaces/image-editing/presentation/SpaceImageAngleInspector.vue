<script setup lang="ts">
import { nextTick, onMounted, toRef, useTemplateRef } from "vue";

import { useImageDerivationActions } from "../../canvas/presentation/images/use-image-derivation-actions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { imageEditOf } from "../image-edit";
import SpaceImageViewpointEditor from "./SpaceImageViewpointEditor.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const emit = defineEmits<{ close: [] }>();
const panel = useTemplateRef<HTMLElement>("panel");
const { busy, availability, help, openAiSettings, run } = useImageDerivationActions({
  node: toRef(props, "node"),
  mode: "angle",
  open: toRef(() => true),
  close: () => emit("close"),
});

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
    <SpaceImageViewpointEditor
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
