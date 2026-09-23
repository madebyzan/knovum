<script setup lang="ts">
import { MessageCircle, ScanLine } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction, showsAction } from "../../../../shared/contracts/action-availability";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";
import SpaceCanvasImageActivePanel from "./SpaceCanvasImageActivePanel.vue";
import SpaceCanvasImageToolTrigger from "./SpaceCanvasImageToolTrigger.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceImageGeneratePanels from "./SpaceImageGeneratePanels.vue";
import SpaceImageLayerPanel from "./SpaceImageLayerPanel.vue";
import SpaceNodeBarCommonActions from "./SpaceNodeBarCommonActions.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useImageNodeBarState } from "./use-image-node-bar-state";
import { useNodeActionBarDensity } from "./use-node-action-bar-density";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node: SpaceCanvasNode<SpaceImagePayload>;
  availableWidth: number;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t, locale } = useI18n();
const {
  comments,
  openPopover,
  activeTool,
  commenting,
  directAvailability,
  directReason,
  dimensionsReady,
  beginTool,
  setPopover,
  beginComment,
  removeBackground,
} = useImageNodeBarState(toRef(props, "node"), toRef(props, "derivations"), () =>
  emit("invokeDerivation", "remove-image-background"),
);
const bar = useTemplateRef<HTMLElement>("bar");
const densityIdentity = computed(
  () => `${locale.value}:${props.node.id}:${activeTool.value ?? ""}:${commenting.value}`,
);
const { compact } = useNodeActionBarDensity(bar, toRef(props, "availableWidth"), densityIdentity);
const commandClass =
  "inline-flex h-(--knovum-control-height-md) shrink-0 items-center gap-1.5 rounded-control px-2.5 text-label text-foreground-secondary outline-none transition-colors duration-150 ease-standard hover:bg-control-hover hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring motion-reduce:transition-none";
</script>

<template>
  <SpaceCanvasImageActivePanel v-if="activeTool" :node="node" />
  <ToolbarRoot
    v-else-if="commenting"
    class="flex h-(--knovum-control-height-lg) w-max items-center rounded-panel border border-border-strong bg-overlay p-px shadow-control"
    :aria-label="t('spaces.media.comment')"
  >
    <ToolbarButton as-child>
      <button type="button" :class="commandClass" aria-pressed="true" @click="comments.cancel()">
        <MessageCircle class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
        {{ t("spaces.media.comment") }}
      </button>
    </ToolbarButton>
  </ToolbarRoot>
  <div v-else ref="bar" class="w-max" :style="{ maxWidth: `${availableWidth}px` }">
    <ToolbarRoot
      class="flex h-(--knovum-control-height-lg) w-max max-w-full cursor-default items-center gap-0.5 overflow-x-auto rounded-panel border border-border-strong bg-overlay p-px shadow-control select-none"
      orientation="horizontal"
      :loop="false"
      :aria-label="t('spaces.canvas.nodeActions.label')"
    >
      <ToolbarButton as-child>
        <button
          type="button"
          :class="commandClass"
          :aria-label="t('spaces.media.comment')"
          :title="compact ? t('spaces.media.comment') : undefined"
          @click="beginComment"
        >
          <MessageCircle class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
          <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{
            t("spaces.media.comment")
          }}</span>
        </button>
      </ToolbarButton>
      <SpaceImageGeneratePanels
        :node="node"
        mode="upscale"
        :compact="compact"
        :open="openPopover === 'upscale'"
        @update:open="setPopover('upscale', $event)"
      />
      <UiTooltip
        v-if="showsAction(directAvailability)"
        :text="directReason"
        :disabled="canRunAction(directAvailability)"
        side="top"
      >
        <ToolbarButton as-child>
          <button
            type="button"
            :class="commandClass"
            :aria-disabled="!canRunAction(directAvailability)"
            :aria-label="t('spaces.creation.removeBackground')"
            :title="compact ? t('spaces.creation.removeBackground') : undefined"
            @click="removeBackground"
          >
            <ScanLine class="size-(--knovum-icon-size-toolbar)" :stroke-width="1.6" />
            <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{
              t("spaces.creation.removeBackground")
            }}</span>
          </button>
        </ToolbarButton>
      </UiTooltip>
      <SpaceCanvasImageToolTrigger mode="eraser" :compact="compact" @begin="beginTool" />
      <SpaceImageLayerPanel
        :node="node"
        :compact="compact"
        :open="openPopover === 'layers'"
        @update:open="setPopover('layers', $event)"
      />
      <SpaceImageGeneratePanels
        :node="node"
        mode="angle"
        :compact="compact"
        :open="openPopover === 'angle'"
        @update:open="setPopover('angle', $event)"
      />
      <SpaceCanvasImageToolTrigger mode="adjust" :compact="compact" @begin="beginTool" />
      <SpaceCanvasImageToolTrigger
        mode="crop"
        :compact="compact"
        :disabled="!dimensionsReady"
        @begin="beginTool"
      />
      <SpaceCanvasImageToolTrigger mode="transform" :compact="compact" @begin="beginTool" />
      <SpaceCanvasSelectionMenu
        :actions="actions"
        :derivations="derivations"
        node-bar
        exclude-direct-image-actions
        @invoke-derivation="emit('invokeDerivation', $event)"
      />
      <SpaceNodeBarCommonActions @inspect="actions.inspect" @download="actions.download" />
    </ToolbarRoot>
  </div>
</template>
