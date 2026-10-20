<script setup lang="ts">
import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";
import SpaceImageDepthAction from "../../../image-editing/presentation/SpaceImageDepthAction.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import SpaceCanvasSelectionMenu from "../selection/SpaceCanvasSelectionMenu.vue";
import type { useCanvasSelectionActions } from "../selection/use-canvas-selection-actions";
import type { ImageActionAvailability } from "./image-action-availability";
import SpaceCanvasImageDerivationActions from "./SpaceCanvasImageDerivationActions.vue";
import SpaceCanvasImageOverflowActions from "./SpaceCanvasImageOverflowActions.vue";
import type { DirectImageDerivation } from "./use-image-node-bar-state";
defineProps<{
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
  runDirect: [actionId: DirectImageDerivation];
}>();
/** The decomposition family has its own controls here, so the creation submenu does not repeat it. */
const FAMILY_DERIVATIONS: readonly CreationIntentId[] = ["remove-image-background", "split-image-layers"];
</script>
<template>
  <SpaceCanvasSelectionMenu
    :actions="actions"
    :derivations="derivations"
    node-bar
    node-tools-visible
    :exclude-actions="FAMILY_DERIVATIONS"
    @invoke-derivation="emit('invokeDerivation', $event)"
  >
    <template #node-tools>
      <SpaceCanvasImageOverflowActions
        v-if="!node.data.imageDepth && minimal"
        :dimensions-ready="dimensionsReady"
        @begin-tool="emit('beginTool', $event)"
        @begin-comment="emit('beginComment')"
      />
      <SpaceCanvasImageDerivationActions
        v-if="minimal"
        menu-item
        :availability-by-derivation="availabilityByDerivation"
        :reason-by-derivation="reasonByDerivation"
        :removal-availability="removalAvailability"
        :removal-reason="removalReason"
        :selection-availability="selectionAvailability"
        :selection-reason="selectionReason"
        @run-direct="emit('runDirect', $event)"
        @begin-removal="emit('beginRemoval')"
        @begin-selection="emit('beginSelection')"
      />
      <SpaceImageDepthAction :node="node" />
    </template>
  </SpaceCanvasSelectionMenu>
</template>
