<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { computed, nextTick, shallowRef, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import type { ImageExecutionTarget } from "../../../contracts/generated/ImageExecutionTarget";
import SpaceCanvasImageEditing from "../../../image-editing/presentation/SpaceCanvasImageEditing.vue";
import SpaceCanvasObjectRemoval from "../../../image-editing/presentation/SpaceCanvasObjectRemoval.vue";
import SpaceImageAngleInspector from "../../../image-editing/presentation/SpaceImageAngleInspector.vue";
import SpaceImageLayersPopover from "../../../image-editing/presentation/SpaceImageLayersPopover.vue";
import SpaceImageSelectionControls from "../../../image-editing/presentation/SpaceImageSelectionControls.vue";
import SpaceImageUpscalePopover from "../../../image-editing/presentation/SpaceImageUpscalePopover.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { useImageSelectionSessionStore } from "../../../knowledge-derivations/image-selection-session-store";
import { hasVisibleCreationActions } from "../../../knowledge-derivations/presentation/visible-derivation-actions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { useNodeReaderAction } from "../../../reading/use-node-reader-action";
import { CANVAS_ACTION_BAR_CLASSES } from "../canvas-action-classes";
import SpaceCanvasNodeAccessActions from "../selection/SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasSelectionAiActions from "../selection/SpaceCanvasSelectionAiActions.vue";
import type { useCanvasSelectionActions } from "../selection/use-canvas-selection-actions";
import { useNodeActionBarDensity } from "../selection/use-node-action-bar-density";
import { useNodePrimaryAction } from "../selection/use-node-primary-action";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
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
  invokeDerivation: [actionId: CreationIntentId, execution?: ImageExecutionTarget];
  "update:angleOpen": [open: boolean];
}>();
const { t, locale } = useI18n();
const nodeRef = toRef(props, "node");
const readerAction = useNodeReaderAction(nodeRef);
const { primary } = useNodePrimaryAction(nodeRef, props.actions);
const barState = useImageNodeBarState(nodeRef, toRef(props, "derivations"), (actionId, execution) =>
  emit("invokeDerivation", actionId, execution),
);
const imageSelection = useImageSelectionSessionStore();
const selecting = computed(() => imageSelection.active?.nodeId === props.node.id);
const hasCreationActions = computed(() => hasVisibleCreationActions(props.derivations));
const bar = useTemplateRef<HTMLElement>("bar");
const densityIdentity = computed(
  () =>
    `${locale.value}:${props.node.id}:${props.actions.state.value.canAsk}:${hasCreationActions.value}:${barState.activeTool.value ?? ""}:${barState.removing.value}:${barState.commenting.value}`,
);
const { compact, minimal, ready } = useNodeActionBarDensity(
  bar,
  toRef(props, "availableWidth"),
  densityIdentity,
);
const restoreFocus = shallowRef(false);
watch(
  [selecting, barState.activeTool, barState.removing, barState.commenting, () => props.angleOpen],
  (current, previous) => {
    if (current.every((value) => !value) && previous.some(Boolean)) restoreFocus.value = true;
  },
);
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
  <SpaceCanvasImageEditing v-else-if="barState.activeTool.value" :node="node" />
  <SpaceCanvasObjectRemoval v-else-if="barState.removing.value" :node="node" />
  <SpaceImageAngleInspector v-else-if="angleOpen" :node="node" @close="emit('update:angleOpen', false)" />
  <SpaceImageCommentModeBar v-else-if="barState.commenting.value" />
  <SpaceImageLayersPopover
    v-else-if="barState.layersChoice.pending.value?.groups?.length"
    :groups="barState.layersChoice.pending.value.groups"
    @run="barState.layersChoice.run"
    @cancel="barState.layersChoice.cancel"
  />
  <div
    v-else
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
      <SpaceCanvasSelectionAiActions
        v-if="!minimal"
        :can-ask="actions.state.value.canAsk"
        :derivations="derivations"
        :compact="compact"
        separator
        @ask="actions.ask"
        @invoke-derivation="emit('invokeDerivation', $event)"
      />
      <SpaceCanvasActionTrigger
        v-if="primary && !minimal"
        :label="primary.label"
        :icon="primary.icon"
        :compact="compact"
        @invoke="primary.run"
      />
      <SpaceCanvasImageEditActions
        v-if="!node.data.imageDepth && !minimal"
        :compact="compact"
        :dimensions-ready="barState.dimensionsReady.value"
        @begin-tool="barState.beginTool"
        @begin-comment="barState.beginComment"
      />
      <SpaceImageUpscalePopover
        :node="node"
        :compact="compact"
        :open="barState.openPopover.value === 'upscale'"
        @update:open="barState.setPopover('upscale', $event)"
      />
      <SpaceCanvasImageDerivationActions
        v-if="!minimal"
        :compact="compact"
        :availability-by-derivation="barState.availabilityByDerivation.value"
        :reason-by-derivation="barState.reasonByDerivation.value"
        :removal-availability="barState.removalAvailability.value"
        :removal-reason="barState.removalReason.value"
        :selection-availability="barState.selectionAvailability.value"
        :selection-reason="barState.selectionReason.value"
        @run-direct="barState.runDirectDerivation"
        @begin-removal="barState.beginRemoval"
        @begin-selection="barState.beginSelection"
      />
      <SpaceCanvasImageMenu
        :node="node"
        :actions="actions"
        :derivations="derivations"
        :minimal="minimal"
        :dimensions-ready="barState.dimensionsReady.value"
        :availability-by-derivation="barState.availabilityByDerivation.value"
        :reason-by-derivation="barState.reasonByDerivation.value"
        :removal-availability="barState.removalAvailability.value"
        :removal-reason="barState.removalReason.value"
        :selection-availability="barState.selectionAvailability.value"
        :selection-reason="barState.selectionReason.value"
        @invoke-derivation="emit('invokeDerivation', $event)"
        @begin-tool="barState.beginTool"
        @begin-comment="barState.beginComment"
        @begin-removal="barState.beginRemoval"
        @begin-selection="barState.beginSelection"
        @begin-angle="barState.beginAngle() && emit('update:angleOpen', true)"
        @run-direct="barState.runDirectDerivation"
      />
      <SpaceCanvasNodeAccessActions
        :export-target="actions.exportTarget.value"
        :reader-action="readerAction"
        @inspect="actions.inspect"
        @save="actions.exportFormat"
      >
        <SpaceCanvasImageComparisonAction :node="node" />
      </SpaceCanvasNodeAccessActions>
    </ToolbarRoot>
  </div>
</template>
