<script setup lang="ts">
import { computed, shallowRef, nextTick, onMounted, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { imageActionAvailability } from "../../canvas/presentation/images/image-action-availability";
import { imageActionHelp } from "../../canvas/presentation/images/image-action-help";
import {
  imageRemovalSize,
  imageRemovalSizeValid,
  useImageSourceDimensionsStore,
} from "../../canvas/presentation/images/image-source-dimensions-store";
import type { ImageExecutionTarget } from "../../contracts/generated/ImageExecutionTarget";
import { useImageRemovalSessionStore } from "../../knowledge-derivations/image-removal-session-store";
import { useImageOperationCapabilities } from "../../knowledge-derivations/use-image-capabilities";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import SpaceImageExecutionSelect from "./SpaceImageExecutionSelect.vue";
import SpaceImageObjectRemovalControls from "./SpaceImageObjectRemovalControls.vue";

const props = defineProps<{ node: SpaceCanvasNode<SpaceImagePayload> }>();
const { t } = useI18n();
const router = useRouter();
const panel = useTemplateRef<HTMLElement>("panel");
const removal = useImageRemovalSessionStore();
const dimensions = useImageSourceDimensionsStore();
const open = computed(() => removal.active?.nodeId === props.node.id);
const capability = useImageOperationCapabilities(open);
const mode = computed(() => removal.active?.mode ?? "mask");
const selectedExecution = shallowRef<ImageExecutionTarget>();
const targetOptions = computed(
  () => capability.data.value?.targets.filter((target) => target.capability === "imageInpainting") ?? [],
);
const target = computed(() =>
  selectedExecution.value
    ? targetOptions.value.find(
        (option) => JSON.stringify(option.execution) === JSON.stringify(selectedExecution.value),
      )
    : targetOptions.value.find((option) => option.selected),
);
const availability = computed(() =>
  imageActionAvailability({
    sourceReady: Boolean(props.node.data.sourceRevision),
    editing: !open.value,
    capability: capability.isFetching.value
      ? undefined
      : mode.value === "mask"
        ? target.value?.availability
        : capability.data.value?.promptRemoval,
    failed: capability.isError.value && !capability.isFetching.value,
    busy: removal.submitting,
  }),
);
const size = computed(() =>
  imageRemovalSize(props.node, dimensions.forNode(props.node.id, props.node.data.sourceRevision)),
);
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
  () => imageActionHelp(availability.value) ?? (hint.value ? { label: hint.value } : undefined),
);

function openAiSettings() {
  removal.cancel();
  void router.push({ name: "settings", query: { pane: "aiModels" } });
}

function generate() {
  if (!canGenerate.value) return;
  void removal
    .generate(mode.value === "mask" ? target.value?.execution : undefined)
    .catch((error) => toast.error(getErrorMessage(error)));
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
    <SpaceImageExecutionSelect
      v-if="mode === 'mask'"
      class="mb-3"
      :options="targetOptions"
      :execution="target?.execution"
      :disabled="removal.submitting"
      @select="selectedExecution = $event"
    />
    <SpaceImageObjectRemovalControls
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
