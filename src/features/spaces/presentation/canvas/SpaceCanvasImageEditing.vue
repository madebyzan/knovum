<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { defaultImageEdit, imageEditOf } from "../../image-editing/image-edit";
import { useImageEditSessionStore } from "../../image-editing/image-edit-session-store";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useImageSourceDimensionsStore } from "./image-source-dimensions-store";
import SpaceImageAdjustmentEditor from "./SpaceImageAdjustmentEditor.vue";
import SpaceImageCropEditor from "./SpaceImageCropEditor.vue";
import SpaceImageTransformControls from "./SpaceImageTransformControls.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const session = useImageEditSessionStore();
const imageDimensions = useImageSourceDimensionsStore();
const panel = useTemplateRef<HTMLElement>("panel");
const active = computed(() => (session.active?.nodeId === props.node.id ? session.active : undefined));
const edit = computed(() => active.value?.draft ?? imageEditOf(props.node.data));
const dimensions = computed(() => {
  const loaded = imageDimensions.forNode(props.node.id, props.node.data.sourceRevision);
  const width = props.node.data.intrinsicWidth ?? loaded?.width;
  const height = props.node.data.intrinsicHeight ?? loaded?.height;
  return width && height ? { width, height } : undefined;
});
function apply() {
  void session.apply().catch((error) => toast.error(getErrorMessage(error)));
}
watch(
  () => active.value?.tool,
  async (tool) => {
    if (!tool) return;
    await nextTick();
    panel.value
      ?.querySelector<HTMLElement>("input:not(:disabled), button:not(:disabled)")
      ?.focus({ preventScroll: true });
  },
  { immediate: true, flush: "post" },
);
</script>

<template>
  <div ref="panel">
    <SpaceImageTransformControls
      v-if="active?.tool === 'transform'"
      :edit="edit"
      :busy="session.applying"
      @update="session.update"
      @cancel="session.cancel"
      @apply="apply"
    />
    <div
      v-else
      class="w-max max-w-[calc(100vw-2rem)] rounded-panel border border-stroke-strong bg-surface-overlay p-3 shadow-popover"
    >
      <SpaceImageAdjustmentEditor
        v-if="active?.tool === 'adjust'"
        :adjustments="edit.adjustments"
        @update="(key, value) => session.update({ adjustments: { ...edit.adjustments, [key]: value } })"
        @reset="session.update({ adjustments: defaultImageEdit().adjustments })"
        @cancel="session.cancel"
        @apply="apply"
      />
      <SpaceImageCropEditor
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
    </div>
  </div>
</template>
