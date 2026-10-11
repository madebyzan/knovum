<script setup lang="ts">
import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";
import SpaceImageLocalActions from "../../../image-editing/presentation/SpaceImageLocalActions.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { canvasBarPlacement } from "../canvas-bar-density-policy";
import SpaceCanvasSelectionMenu from "../selection/SpaceCanvasSelectionMenu.vue";
import type { useCanvasSelectionActions } from "../selection/use-canvas-selection-actions";
import type { ImageActionAvailability } from "./image-action-availability";
import SpaceCanvasImageOverflowActions from "./SpaceCanvasImageOverflowActions.vue";
import SpaceCanvasImageRemoveObjectTrigger from "./SpaceCanvasImageRemoveObjectTrigger.vue";
defineProps<{
  node: SpaceCanvasNode<SpaceImagePayload>;
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  minimal: boolean;
  dimensionsReady: boolean;
  removalAvailability: ImageActionAvailability;
  removalReason: string;
}>();
const emit = defineEmits<{
  invokeDerivation: [actionId: CreationIntentId];
  beginTool: [tool: ImageEditTool];
  beginComment: [];
  beginRemoval: [];
}>();
</script>
<template>
  <SpaceCanvasSelectionMenu
    :actions="actions"
    :derivations="derivations"
    node-bar
    node-tools-visible
    :exclude-direct-image-actions="canvasBarPlacement('image-remove-background', minimal) === 'bar'"
    @invoke-derivation="emit('invokeDerivation', $event)"
  >
    <template #node-tools>
      <SpaceImageLocalActions :node="node" />
      <SpaceCanvasImageOverflowActions
        v-if="!node.data.imageDepth && canvasBarPlacement('image-edit', minimal) === 'menu'"
        :dimensions-ready="dimensionsReady"
        @begin-tool="emit('beginTool', $event)"
        @begin-comment="emit('beginComment')"
      />
      <SpaceCanvasImageRemoveObjectTrigger
        v-if="canvasBarPlacement('image-remove-object', minimal) === 'menu'"
        menu-item
        :disabled="removalAvailability.state !== 'ready'"
        :reason="removalReason"
        @begin="emit('beginRemoval')"
      />
    </template>
  </SpaceCanvasSelectionMenu>
</template>
