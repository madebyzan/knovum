<script setup lang="ts">
import { computed, nextTick, onMounted, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { errorMessage } from "../../../../foundation/errors/error-format";
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
const router = useRouter();
const panel = useTemplateRef<HTMLElement>("panel");
const imageDimensions = useImageSourceDimensions();
const removal = useImageRemovalSession();
const open = computed(() => removal.active?.nodeId === props.node.id);
const capability = useImageOperationCapabilities(open);
const mode = computed(() => removal.active?.mode ?? "mask");
const availability = computed(() =>
  imageActionAvailability({
    sourceReady: Boolean(props.node.data.sourceRevision),
    editing: !open.value,
    capability: capability.isFetching.value
      ? undefined
      : mode.value === "mask"
        ? capability.data.value?.maskedRemoval
        : capability.data.value?.promptRemoval,
    failed: capability.isError.value && !capability.isFetching.value,
    busy: removal.submitting,
  }),
);
const size = computed(() => imageRemovalSize(props.node, imageDimensions));
const sizeValid = computed(() => imageRemovalSizeValid(size.value));
const canGenerate = computed(
  () =>
    availability.value.state === "ready" &&
    (mode.value === "mask"
      ? Boolean(removal.active?.strokes.length) && sizeValid.value
      : Boolean(removal.active?.prompt.trim())),
);
const hint = computed(() => {
  if (removal.submitting) return t("spaces.media.imageEditor.removeObject.submitting");
  if (mode.value !== "mask") return undefined;
  if (!size.value) return t("spaces.media.imageEditor.status.imagePreparing");
  if (!sizeValid.value) return t("spaces.media.imageEditor.removeObject.sizeLimit");
  if (!removal.canDraw) return t("spaces.media.imageEditor.removeObject.limitReached");
  return undefined;
});
const help = computed(
  () => imageActionHelp(availability.value, t) ?? (hint.value ? { label: hint.value } : undefined),
);

function openAiSettings() {
  removal.cancel();
  void router.push({ name: "settings", query: { pane: "aiModels" } });
}

function generate() {
  if (!canGenerate.value) return;
  void removal.generate().catch((error) => toast.error(errorMessage(error)));
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
      :mode="mode"
      :prompt="removal.active?.prompt ?? ''"
      :diameter="removal.brushDiameter"
      :can-undo="Boolean(removal.active?.strokes.length)"
      :submitting="removal.submitting"
      :can-generate="canGenerate"
      :help="help"
      @update-mode="removal.setMode"
      @update-prompt="removal.setPrompt"
      @update-diameter="removal.setBrushDiameter"
      @undo="removal.undoStroke"
      @cancel="removal.cancel"
      @generate="generate"
      @open-settings="openAiSettings"
    />
  </div>
</template>
