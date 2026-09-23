<script setup lang="ts">
import { Download, MessageCircle, ScanLine } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed, onBeforeUnmount, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { availableWhen, canRunAction } from "../../../../shared/contracts/action-availability";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { useImageCommentSession } from "../../comments/image-comment-session";
import { useImageEditSession } from "../../image-editing/image-edit-session";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { imageActionHelp } from "./image-action-help";
import SpaceCanvasImageToolPanels from "./SpaceCanvasImageToolPanels.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceImageGeneratePanels from "./SpaceImageGeneratePanels.vue";
import SpaceImageLayerPanel from "./SpaceImageLayerPanel.vue";
import SpaceImageTransformPanel from "./SpaceImageTransformPanel.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node: SpaceCanvasNode<SpaceImagePayload>;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const imageEditSession = useImageEditSession();
const comments = useImageCommentSession();
const directAvailability = computed(() =>
  availableWhen(imageEditSession.active?.nodeId !== props.node.id, "finish_image_edit"),
);
const directReason = computed(() => imageActionHelp(directAvailability.value, t)?.label ?? "");
const transformEdit = computed(() => {
  const active = imageEditSession.active;
  return active?.nodeId === props.node.id && active.tool === "transform" ? active.draft : null;
});
watch(
  () => props.node.id,
  (_next, previous) => {
    if (imageEditSession.active?.nodeId === previous) imageEditSession.cancel();
    if (comments.activeNodeId === previous) comments.cancel();
  },
);
onBeforeUnmount(() => {
  if (imageEditSession.active?.nodeId === props.node.id) imageEditSession.cancel();
  if (comments.activeNodeId === props.node.id) comments.cancel();
});
watch(
  () => imageEditSession.active?.nodeId,
  (nodeId) => {
    if (nodeId && comments.activeNodeId === nodeId) comments.cancel();
  },
);
function applyTransform() {
  void imageEditSession.apply().catch((error) => toast.error(String(error)));
}
const commandClass =
  "inline-flex h-(--knovum-control-height-md) shrink-0 items-center gap-1.5 rounded-control px-2.5 text-label text-foreground-secondary outline-none transition-colors duration-100 hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring disabled:cursor-not-allowed disabled:opacity-45";
</script>

<template>
  <SpaceImageTransformPanel
    v-if="transformEdit"
    :edit="transformEdit"
    :busy="imageEditSession.applying"
    @update="imageEditSession.update"
    @cancel="imageEditSession.cancel"
    @apply="applyTransform"
  />
  <ToolbarRoot
    v-else
    class="flex h-(--knovum-control-height-lg) w-max max-w-[calc(100vw-2rem)] cursor-default items-center gap-0.5 overflow-x-auto rounded-panel border border-border-strong bg-overlay p-px shadow-control select-none"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.canvas.nodeActions.label')"
  >
    <ToolbarButton as-child>
      <button
        type="button"
        :class="[commandClass, comments.activeNodeId === node.id && 'bg-control-hover text-foreground']"
        :aria-pressed="comments.activeNodeId === node.id"
        :disabled="!canRunAction(directAvailability)"
        @click="comments.activeNodeId === node.id ? comments.cancel() : comments.begin(node.id)"
      >
        <MessageCircle class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
        {{ t("spaces.media.comment") }}
      </button>
    </ToolbarButton>
    <SpaceImageGeneratePanels :node="node" mode="upscale" />
    <UiTooltip
      v-if="canRunAction(derivations['remove-image-background'])"
      :text="directReason"
      :disabled="canRunAction(directAvailability)"
      side="top"
    >
      <ToolbarButton as-child>
        <button
          type="button"
          :class="[commandClass, 'aria-disabled:opacity-45']"
          :aria-disabled="!canRunAction(directAvailability)"
          @click="canRunAction(directAvailability) && emit('invokeDerivation', 'remove-image-background')"
        >
          <ScanLine class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          {{ t("spaces.creation.removeBackground") }}
        </button>
      </ToolbarButton>
    </UiTooltip>
    <SpaceCanvasImageToolPanels :node="node" mode="eraser" />
    <SpaceImageLayerPanel :node="node" />
    <SpaceImageGeneratePanels :node="node" mode="angle" />
    <SpaceCanvasImageToolPanels :node="node" mode="adjust" />
    <SpaceCanvasImageToolPanels :node="node" mode="crop" />
    <SpaceCanvasImageToolPanels :node="node" mode="transform" />
    <SpaceCanvasSelectionMenu
      :actions="actions"
      :derivations="derivations"
      node-bar
      exclude-direct-image-actions
      @invoke-derivation="emit('invokeDerivation', $event)"
    />
    <div class="flex h-full items-center border-l border-divider pl-0.5">
      <UiTooltip :text="directReason" :disabled="canRunAction(directAvailability)" side="top">
        <ToolbarButton as-child>
          <button
            type="button"
            :class="[commandClass, 'font-medium text-foreground aria-disabled:opacity-45']"
            :aria-disabled="!canRunAction(directAvailability)"
            @click="canRunAction(directAvailability) && actions.download()"
          >
            <Download class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
            {{ t("spaces.canvas.nodeActions.download") }}
          </button>
        </ToolbarButton>
      </UiTooltip>
    </div>
  </ToolbarRoot>
</template>
