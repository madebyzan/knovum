<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { computed, nextTick, shallowRef, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import SpaceCanvasImageEditing from "../../../image-editing/presentation/SpaceCanvasImageEditing.vue";
import SpaceCanvasObjectRemoval from "../../../image-editing/presentation/SpaceCanvasObjectRemoval.vue";
import SpaceImageAngleInspector from "../../../image-editing/presentation/SpaceImageAngleInspector.vue";
import SpaceImageSelectionControls from "../../../image-editing/presentation/SpaceImageSelectionControls.vue";
import SpaceImageUpscalePopover from "../../../image-editing/presentation/SpaceImageUpscalePopover.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { useImageSelectionSessionStore } from "../../../knowledge-derivations/image-selection-session-store";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { CANVAS_ACTION_BAR_CLASSES } from "../canvas-action-classes";
import SpaceCanvasNodeAccessActions from "../selection/SpaceCanvasNodeAccessActions.vue";
import type { useCanvasSelectionActions } from "../selection/use-canvas-selection-actions";
import { useNodeActionBarDensity } from "../selection/use-node-action-bar-density";
import SpaceCanvasImageAngleTrigger from "./SpaceCanvasImageAngleTrigger.vue";
import SpaceCanvasImageComparisonAction from "./SpaceCanvasImageComparisonAction.vue";
import SpaceCanvasImageDerivationActions from "./SpaceCanvasImageDerivationActions.vue";
import SpaceCanvasImageEditActions from "./SpaceCanvasImageEditActions.vue";
import SpaceCanvasImageMenu from "./SpaceCanvasImageMenu.vue";
import SpaceImageCommentModeBar from "./SpaceImageCommentModeBar.vue";
import { useImageNodeBarState } from "./use-image-node-bar-state";
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
  availabilityByDerivation,
  reasonByDerivation,
  dimensionsReady,
  removalAvailability,
  removalReason,
  selectionAvailability,
  selectionReason,
  beginTool,
  beginRemoval,
  beginSelection,
  beginAngle,
  setPopover,
  beginComment,
  runDirectDerivation,
} = useImageNodeBarState(toRef(props, "node"), toRef(props, "derivations"), (actionId) =>
  emit("invokeDerivation", actionId),
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
        v-if="!node.data.imageDepth && !minimal"
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
      <SpaceCanvasImageDerivationActions
        v-if="!minimal"
        :compact="compact"
        :availability-by-derivation="availabilityByDerivation"
        :reason-by-derivation="reasonByDerivation"
        :removal-availability="removalAvailability"
        :removal-reason="removalReason"
        :selection-availability="selectionAvailability"
        :selection-reason="selectionReason"
        @run-direct="runDirectDerivation"
        @begin-removal="beginRemoval"
        @begin-selection="beginSelection"
      />
      <SpaceCanvasImageAngleTrigger
        :compact="compact"
        @begin="beginAngle() && emit('update:angleOpen', true)"
      />
      <SpaceCanvasImageComparisonAction :node="node" :compact="compact" />
      <SpaceCanvasImageMenu
        :node="node"
        :actions="actions"
        :derivations="derivations"
        :minimal="minimal"
        :dimensions-ready="dimensionsReady"
        :availability-by-derivation="availabilityByDerivation"
        :reason-by-derivation="reasonByDerivation"
        :removal-availability="removalAvailability"
        :removal-reason="removalReason"
        :selection-availability="selectionAvailability"
        :selection-reason="selectionReason"
        @invoke-derivation="emit('invokeDerivation', $event)"
        @begin-tool="beginTool"
        @begin-comment="beginComment"
        @begin-removal="beginRemoval"
        @begin-selection="beginSelection"
        @run-direct="runDirectDerivation"
      />
      <SpaceCanvasNodeAccessActions
        :export-target="actions.exportTarget.value"
        @inspect="actions.inspect"
        @save="actions.exportFormat"
      />
    </ToolbarRoot>
  </div>
</template>
