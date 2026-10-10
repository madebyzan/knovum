<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { computed, nextTick, shallowRef, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import { useImageSelectionSessionStore } from "../../knowledge-derivations/image-selection-session-store";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { CANVAS_ACTION_BAR_CLASSES } from "./canvas-action-classes";
import { canvasBarPlacement } from "./canvas-bar-density-policy";
import SpaceCanvasImageAngleTrigger from "./SpaceCanvasImageAngleTrigger.vue";
import SpaceCanvasImageEditActions from "./SpaceCanvasImageEditActions.vue";
import SpaceCanvasImageEditing from "./SpaceCanvasImageEditing.vue";
import SpaceCanvasImageMenu from "./SpaceCanvasImageMenu.vue";
import SpaceCanvasImageRemoveBackgroundTrigger from "./SpaceCanvasImageRemoveBackgroundTrigger.vue";
import SpaceCanvasImageRemoveObjectTrigger from "./SpaceCanvasImageRemoveObjectTrigger.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasObjectRemoval from "./SpaceCanvasObjectRemoval.vue";
import SpaceImageAngleInspector from "./SpaceImageAngleInspector.vue";
import SpaceImageCommentModeBar from "./SpaceImageCommentModeBar.vue";
import SpaceImageLayerGeneration from "./SpaceImageLayerGeneration.vue";
import SpaceImageSelectionControls from "./SpaceImageSelectionControls.vue";
import SpaceImageUpscalePopover from "./SpaceImageUpscalePopover.vue";
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
const imageSelection = useImageSelectionSessionStore();
const selecting = computed(() => imageSelection.active?.nodeId === props.node.id);
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
watch([selecting, activeTool, removing, commenting, () => props.angleOpen], (current, previous) => {
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
  <SpaceImageSelectionControls v-if="selecting" />
  <SpaceCanvasImageEditing v-else-if="activeTool" :node="node" />
  <SpaceCanvasObjectRemoval v-else-if="removing" :node="node" />
  <SpaceImageAngleInspector v-else-if="angleOpen" :node="node" @close="emit('update:angleOpen', false)" />
  <SpaceImageCommentModeBar v-else-if="commenting" />
  <div
    v-if="!selecting && !activeTool && !removing && !commenting && !angleOpen"
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
        v-if="!node.data.imageDepth && canvasBarPlacement('image-edit', minimal) === 'bar'"
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
      <SpaceImageLayerGeneration
        :node="node"
        :compact="compact"
        :open="openPopover === 'layers'"
        @update:open="setPopover('layers', $event)"
      />
      <SpaceCanvasImageAngleTrigger
        :compact="compact"
        @begin="beginAngle() && emit('update:angleOpen', true)"
      />
      <SpaceCanvasImageMenu
        :node="node"
        :actions="actions"
        :derivations="derivations"
        :minimal="minimal"
        :dimensions-ready="dimensionsReady"
        :removal-availability="removalAvailability"
        :removal-reason="removalReason"
        @invoke-derivation="emit('invokeDerivation', $event)"
        @begin-tool="beginTool"
        @begin-comment="beginComment"
        @begin-removal="beginRemoval"
      />
      <SpaceCanvasNodeAccessActions @inspect="actions.inspect" @export="actions.exportSelection" />
    </ToolbarRoot>
  </div>
</template>
