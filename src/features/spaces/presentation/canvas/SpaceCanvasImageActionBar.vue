<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { computed, nextTick, shallowRef, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { CANVAS_ACTION_BAR_CLASSES } from "./canvas-action-classes";
import { canvasBarPlacement } from "./canvas-bar-density-policy";
import SpaceCanvasImageActivePanel from "./SpaceCanvasImageActivePanel.vue";
import SpaceCanvasImageAngleTrigger from "./SpaceCanvasImageAngleTrigger.vue";
import SpaceCanvasImageEditActions from "./SpaceCanvasImageEditActions.vue";
import SpaceCanvasImageOverflowActions from "./SpaceCanvasImageOverflowActions.vue";
import SpaceCanvasImageRemovalPanel from "./SpaceCanvasImageRemovalPanel.vue";
import SpaceCanvasImageRemoveBackgroundTrigger from "./SpaceCanvasImageRemoveBackgroundTrigger.vue";
import SpaceCanvasImageRemoveObjectTrigger from "./SpaceCanvasImageRemoveObjectTrigger.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceImageAngleInspector from "./SpaceImageAngleInspector.vue";
import SpaceImageCommentModeBar from "./SpaceImageCommentModeBar.vue";
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
  removing,
  commenting,
  directAvailability,
  directReason,
  dimensionsReady,
  removalAvailability,
  removalReason,
  beginTool,
  beginRemoval,
  beginAngle,
  setPopover,
  beginComment,
  removeBackground,
} = useImageNodeBarState(toRef(props, "node"), toRef(props, "derivations"), () =>
  emit("invokeDerivation", "remove-image-background"),
);
const bar = useTemplateRef<HTMLElement>("bar");
const densityIdentity = computed(
  () => `${locale.value}:${props.node.id}:${activeTool.value ?? ""}:${removing.value}:${commenting.value}`,
);
const { compact, minimal, ready } = useNodeActionBarDensity(
  bar,
  toRef(props, "availableWidth"),
  densityIdentity,
);
const restoreFocus = shallowRef(false);
watch([activeTool, removing, commenting, () => props.angleOpen], (current, previous) => {
  if (current.every((value) => !value) && previous.some(Boolean)) restoreFocus.value = true;
});
watch(
  [ready, restoreFocus],
  async ([isReady, shouldRestore]) => {
    if (!isReady || !shouldRestore) return;
    await nextTick();
    bar.value?.querySelector<HTMLElement>("button")?.focus({ preventScroll: true });
    restoreFocus.value = false;
  },
  { flush: "post" },
);
</script>
<template>
  <SpaceCanvasImageActivePanel v-if="activeTool" :node="node" />
  <SpaceCanvasImageRemovalPanel v-else-if="removing" :node="node" />
  <SpaceImageAngleInspector v-else-if="angleOpen" :node="node" @close="emit('update:angleOpen', false)" />
  <SpaceImageCommentModeBar v-else-if="commenting" />
  <div
    v-if="!activeTool && !removing && !commenting && !angleOpen"
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
      <SpaceCanvasImageRemoveBackgroundTrigger
        v-if="canvasBarPlacement('image-remove-background', minimal) === 'bar'"
        :compact="compact"
        :availability="directAvailability"
        :reason="directReason"
        @begin="removeBackground"
      />
      <SpaceCanvasImageRemoveObjectTrigger
        v-if="canvasBarPlacement('image-remove-object', minimal) === 'bar'"
        :compact="compact"
        :disabled="removalAvailability.state !== 'ready'"
        :reason="removalReason"
        @begin="beginRemoval"
      />
      <SpaceImageLayerPanel
        :node="node"
        :compact="compact"
        :open="openPopover === 'layers'"
        @update:open="setPopover('layers', $event)"
      />
      <SpaceCanvasImageAngleTrigger
        :compact="compact"
        @begin="beginAngle() && emit('update:angleOpen', true)"
      />
      <SpaceCanvasSelectionMenu
        :actions="actions"
        :derivations="derivations"
        node-bar
        :node-tools-visible="
          canvasBarPlacement('image-edit', minimal) === 'menu' ||
          canvasBarPlacement('image-remove-object', minimal) === 'menu'
        "
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
          <SpaceCanvasImageRemoveObjectTrigger
            v-if="canvasBarPlacement('image-remove-object', minimal) === 'menu'"
            menu-item
            :disabled="removalAvailability.state !== 'ready'"
            :reason="removalReason"
            @begin="beginRemoval"
          />
        </template>
      </SpaceCanvasSelectionMenu>
      <SpaceNodeBarCommonActions @inspect="actions.inspect" @download="actions.download" />
    </ToolbarRoot>
  </div>
</template>
