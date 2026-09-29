<script setup lang="ts">
import { Box, ScanLine } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed, nextTick, shallowRef, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { canRunAction, showsAction } from "../../../../shared/contracts/action-availability";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "./canvas-action-classes";
import { canvasBarPlacement } from "./canvas-bar-density-policy";
import { nodeActionLabelClass, nodeActionLabelState } from "./node-action-label";
import SpaceCanvasImageActivePanel from "./SpaceCanvasImageActivePanel.vue";
import SpaceCanvasImageEditActions from "./SpaceCanvasImageEditActions.vue";
import SpaceCanvasImageOverflowActions from "./SpaceCanvasImageOverflowActions.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceImageAngleInspector from "./SpaceImageAngleInspector.vue";
import SpaceImageLayerPanel from "./SpaceImageLayerPanel.vue";
import SpaceImageUpscalePopover from "./SpaceImageUpscalePopover.vue";
import SpaceNodeBarCommonActions from "./SpaceNodeBarCommonActions.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useImageNodeBarState } from "./use-image-node-bar-state";
import { useNodeActionBarDensity } from "./use-node-action-bar-density";
const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node: SpaceCanvasNode<SpaceImagePayload>;
  availableWidth: number;
  angleOpen: boolean;
}>();
const emit = defineEmits<{
  invokeDerivation: [actionId: CreationIntentId];
  "update:angleOpen": [open: boolean];
}>();
const { t, locale } = useI18n();
const {
  openPopover,
  activeTool,
  commenting,
  directAvailability,
  directReason,
  dimensionsReady,
  beginTool,
  beginAngle,
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
const { compact, minimal, ready } = useNodeActionBarDensity(
  bar,
  toRef(props, "availableWidth"),
  densityIdentity,
);
const restoreFocus = shallowRef(false);
watch([activeTool, commenting], ([tool, comment], [previousTool, previousComment]) => {
  if (!tool && !comment && (previousTool || previousComment)) restoreFocus.value = true;
});
watch(
  () => props.angleOpen,
  async (open, previous) => {
    if (open || !previous) return;
    await nextTick();
    bar.value?.querySelector<HTMLElement>("button")?.focus({ preventScroll: true });
  },
);
watch(ready, async (isReady) => {
  if (!isReady || !restoreFocus.value) return;
  await nextTick();
  bar.value?.querySelector<HTMLElement>("button")?.focus({ preventScroll: true });
  restoreFocus.value = false;
});
</script>
<template>
  <SpaceCanvasImageActivePanel v-if="activeTool" :node="node" />
  <SpaceImageAngleInspector
    v-else-if="angleOpen"
    :node="node"
    @close="emit('update:angleOpen', false)"
  />
  <div
    v-if="!activeTool && !commenting && !angleOpen"
    ref="bar"
    class="w-max"
    :class="ready ? 'visible' : 'invisible'"
    :style="{ maxWidth: `${availableWidth}px` }"
  >
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      orientation="horizontal"
      :loop="false"
      :aria-label="t('spaces.canvas.nodeActions.label')"
    >
      <SpaceCanvasImageEditActions
        v-if="canvasBarPlacement('image-edit', minimal) === 'bar'"
        :compact="compact"
        :dimensions-ready="dimensionsReady"
        @begin-tool="beginTool"
        @begin-comment="beginComment"
      />
      <SpaceImageUpscalePopover
        :node="node"
        :compact="compact"
        :open="openPopover === 'upscale'"
        @update:open="setPopover('upscale', $event)"
      />
      <UiTooltip
        v-if="
          canvasBarPlacement('image-remove-background', minimal) === 'bar' && showsAction(directAvailability)
        "
        :text="canRunAction(directAvailability) ? t('spaces.creation.image.removeBackground') : directReason"
        :disabled="canRunAction(directAvailability) && !compact"
        side="top"
      >
        <ToolbarButton as-child>
          <button
            type="button"
            :class="CANVAS_ACTION_BUTTON_CLASSES"
            :aria-disabled="!canRunAction(directAvailability)"
            :aria-label="t('spaces.creation.image.removeBackground')"
            @click="removeBackground"
          >
            <ScanLine class="size-4.5" :stroke-width="1.6" />
            <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{
              t("spaces.creation.image.removeBackground")
            }}</span>
          </button>
        </ToolbarButton>
      </UiTooltip>
      <SpaceImageLayerPanel
        :node="node"
        :compact="compact"
        :open="openPopover === 'layers'"
        @update:open="setPopover('layers', $event)"
      />
      <ToolbarButton as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :aria-label="t('spaces.media.imageEditor.angle.title')"
          :title="compact ? t('spaces.media.imageEditor.angle.title') : undefined"
          @click="beginAngle() && emit('update:angleOpen', true)"
        >
          <Box class="size-4.5" :stroke-width="1.6" />
          <span :class="[nodeActionLabelClass, nodeActionLabelState(compact)]">{{
            t("spaces.media.imageEditor.angle.title")
          }}</span>
        </button>
      </ToolbarButton>
      <SpaceCanvasSelectionMenu
        :actions="actions"
        :derivations="derivations"
        node-bar
        :node-tools-visible="canvasBarPlacement('image-edit', minimal) === 'menu'"
        :exclude-direct-image-actions="canvasBarPlacement('image-remove-background', minimal) === 'bar'"
        @invoke-derivation="emit('invokeDerivation', $event)"
      >
        <template #node-tools>
          <SpaceCanvasImageOverflowActions
            v-if="canvasBarPlacement('image-edit', minimal) === 'menu'"
            :dimensions-ready="dimensionsReady"
            @begin-tool="beginTool"
            @begin-comment="beginComment"
          />
        </template>
      </SpaceCanvasSelectionMenu>
      <SpaceNodeBarCommonActions @inspect="actions.inspect" @download="actions.download" />
    </ToolbarRoot>
  </div>
</template>
