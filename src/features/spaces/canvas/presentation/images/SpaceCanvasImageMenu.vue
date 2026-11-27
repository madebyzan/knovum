<script setup lang="ts">
import { toRef } from "vue";

import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { ImageExecutionTarget } from "../../../contracts/generated/ImageExecutionTarget";
import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";
import SpaceImageDepthAction from "../../../image-editing/presentation/SpaceImageDepthAction.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import SpaceCanvasSelectionAiActions from "../selection/SpaceCanvasSelectionAiActions.vue";
import SpaceCanvasSelectionMenu from "../selection/SpaceCanvasSelectionMenu.vue";
import type { useCanvasSelectionActions } from "../selection/use-canvas-selection-actions";
import { useNodePrimaryAction } from "../selection/use-node-primary-action";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import type { ImageActionAvailability } from "./image-action-availability";
import SpaceCanvasImageAngleTrigger from "./SpaceCanvasImageAngleTrigger.vue";
import SpaceCanvasImageDerivationActions from "./SpaceCanvasImageDerivationActions.vue";
import SpaceCanvasImageOverflowActions from "./SpaceCanvasImageOverflowActions.vue";
import type { DirectImageDerivation } from "./use-image-node-bar-state";

const props = defineProps<{
  node: SpaceCanvasNode<SpaceImagePayload>;
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  minimal: boolean;
  dimensionsReady: boolean;
  availabilityByDerivation: Record<DirectImageDerivation, ImageActionAvailability>;
  reasonByDerivation: Record<DirectImageDerivation, string>;
  removalAvailability: ImageActionAvailability;
  removalReason: string;
  selectionAvailability: ImageActionAvailability;
  selectionReason: string;
}>();
const emit = defineEmits<{
  invokeDerivation: [actionId: CreationIntentId];
  beginTool: [tool: ImageEditTool];
  beginComment: [];
  beginRemoval: [];
  beginSelection: [];
  beginAngle: [];
  runDirect: [actionId: DirectImageDerivation, execution?: ImageExecutionTarget];
}>();
const { primary } = useNodePrimaryAction(toRef(props, "node"), props.actions);
</script>

<template>
  <SpaceCanvasSelectionMenu :actions="actions" node-tools-visible>
    <template #node-tools>
      <SpaceCanvasSelectionAiActions
        v-if="minimal"
        menu-item
        :can-ask="actions.state.value.canAsk"
        :derivations="derivations"
        @ask="actions.ask"
        @invoke-derivation="emit('invokeDerivation', $event)"
      />
      <SpaceCanvasActionTrigger
        v-if="minimal && primary"
        :label="primary.label"
        :icon="primary.icon"
        menu-item
        @invoke="primary.run"
      />
      <SpaceCanvasImageOverflowActions
        v-if="!node.data.imageDepth"
        :dimensions-ready="dimensionsReady"
        :include-primary="minimal"
        @begin-tool="emit('beginTool', $event)"
        @begin-comment="emit('beginComment')"
      />
      <UiSeparator v-if="!node.data.imageDepth" class="my-1" />
      <SpaceCanvasImageDerivationActions
        menu-item
        :include-primary="minimal"
        :availability-by-derivation="availabilityByDerivation"
        :reason-by-derivation="reasonByDerivation"
        :removal-availability="removalAvailability"
        :removal-reason="removalReason"
        :selection-availability="selectionAvailability"
        :selection-reason="selectionReason"
        @run-direct="(actionId, execution) => emit('runDirect', actionId, execution)"
        @begin-removal="emit('beginRemoval')"
        @begin-selection="emit('beginSelection')"
      />
      <SpaceCanvasImageAngleTrigger menu-item @begin="emit('beginAngle')" />
      <SpaceImageDepthAction :node="node" />
    </template>
  </SpaceCanvasSelectionMenu>
</template>
