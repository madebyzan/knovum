<script setup lang="ts">
import { computed } from "vue";
import { toast } from "vue-sonner";

import { defaultImageEdit, imageEditOf } from "../../image-editing/image-edit";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import SpaceImageAdjustPanel from "./SpaceImageAdjustPanel.vue";
import SpaceImageCropPanel from "./SpaceImageCropPanel.vue";
import SpaceImageEraserPanel from "./SpaceImageEraserPanel.vue";
import SpaceImageTransformPanel from "./SpaceImageTransformPanel.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const session = useImageEditSession();
const active = computed(() => (session.active?.nodeId === props.node.id ? session.active : undefined));
const edit = computed(() => active.value?.draft ?? imageEditOf(props.node.data));
const dimensions = computed(() => {
  const loaded = session.dimensionsFor(props.node.id);
  const width = props.node.data.intrinsicWidth ?? loaded?.width;
  const height = props.node.data.intrinsicHeight ?? loaded?.height;
  return width && height ? { width, height } : undefined;
});

function apply() {
  void session.apply().catch((error) => toast.error(String(error)));
}
</script>

<template>
  <SpaceImageTransformPanel
    v-if="active?.tool === 'transform'"
    :edit="edit"
    :busy="session.applying"
    @update="session.update"
    @cancel="session.cancel"
    @apply="apply"
  />
  <div
    v-else
    class="w-max max-w-[calc(100vw-2rem)] rounded-panel border border-border-strong bg-overlay p-3 shadow-popover"
  >
    <SpaceImageAdjustPanel
      v-if="active?.tool === 'adjust'"
      :adjustments="edit.adjustments"
      @update="(key, value) => session.update({ adjustments: { ...edit.adjustments, [key]: value } })"
      @reset="session.update({ adjustments: defaultImageEdit().adjustments })"
      @cancel="session.cancel"
      @apply="apply"
    />
    <SpaceImageCropPanel
      v-else-if="active?.tool === 'crop' && dimensions"
      :crop="edit.crop"
      :source-width="dimensions.width"
      :source-height="dimensions.height"
      :locked-aspect="session.cropAspectRatio"
      @update="(crop) => session.update({ crop })"
      @update-locked-aspect="session.setCropAspectRatio"
      @cancel="session.cancel"
      @apply="apply"
    />
    <SpaceImageEraserPanel
      v-else-if="active?.tool === 'eraser'"
      :diameter="session.brushDiameter"
      :can-undo="edit.eraserStrokes.length > imageEditOf(node.data).eraserStrokes.length"
      :can-apply="edit.eraserStrokes.length !== imageEditOf(node.data).eraserStrokes.length"
      @update-diameter="session.setBrushDiameter"
      @undo="session.undoEraserStroke"
      @cancel="session.cancel"
      @apply="apply"
    />
  </div>
</template>
