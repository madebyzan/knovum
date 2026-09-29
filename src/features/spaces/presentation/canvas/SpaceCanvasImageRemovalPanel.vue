<script setup lang="ts">
import { computed, nextTick, onMounted, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { useImageOperationCapabilities } from "../../knowledge-derivations/image-capabilities";
import { useImageRemovalSession } from "../../knowledge-derivations/image-removal-session";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { imageActionAvailability } from "./image-action-availability";
import { imageActionHelp } from "./image-action-help";
import { imageRemovalSize, imageRemovalSizeValid, useImageSourceDimensions } from "./image-source-dimensions";
import SpaceImageRemoveObjectPanel from "./SpaceImageRemoveObjectPanel.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const { t } = useI18n();
const panel = useTemplateRef<HTMLElement>("panel");
const imageDimensions = useImageSourceDimensions();
const removal = useImageRemovalSession();
const open = computed(() => removal.active?.nodeId === props.node.id);
const capability = useImageOperationCapabilities(open);
const availability = computed(() =>
  imageActionAvailability({
    sourceReady: Boolean(props.node.data.sourceRevision),
    editing: false,
    capability: capability.isFetching.value ? undefined : capability.data.value?.maskedRemoval,
    failed: capability.isError.value && !capability.isFetching.value,
    busy: false,
  }),
);
const size = computed(() => imageRemovalSize(props.node, imageDimensions));
const sizeValid = computed(() => imageRemovalSizeValid(size.value));
const hint = computed(() =>
  removal.submitting
    ? t("spaces.media.imageEditor.removeObject.submitting")
    : !size.value
      ? t("spaces.media.imageEditor.status.imagePreparing")
      : !sizeValid.value
        ? t("spaces.media.imageEditor.removeObject.sizeLimit")
        : !removal.canDraw
          ? t("spaces.media.imageEditor.removeObject.limitReached")
          : imageActionHelp(availability.value, t)?.label,
);

function generate() {
  void removal.generate().catch((error) => toast.error(String(error)));
}
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
    <SpaceImageRemoveObjectPanel
      :diameter="removal.brushDiameter"
      :can-undo="Boolean(removal.active?.strokes.length)"
      :submitting="removal.submitting"
      :can-generate="Boolean(removal.active?.strokes.length) && sizeValid && availability.state === 'ready'"
      :hint="hint"
      @update-diameter="removal.setBrushDiameter"
      @undo="removal.undoStroke"
      @cancel="removal.cancel"
      @generate="generate"
    />
  </div>
</template>
