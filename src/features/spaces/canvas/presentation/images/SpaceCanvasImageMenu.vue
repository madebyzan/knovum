<script setup lang="ts">
import { computed } from "vue";

import type { ImageEditTool } from "../../../image-editing/image-edit-session-store";
import SpaceImageLocalActions from "../../../image-editing/presentation/SpaceImageLocalActions.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../../model/nodes/node-payload";
import { canvasBarPlacement, type CanvasBarAction } from "../canvas-bar-density-policy";
import SpaceCanvasSelectionMenu from "../selection/SpaceCanvasSelectionMenu.vue";
import type { useCanvasSelectionActions } from "../selection/use-canvas-selection-actions";
import type { ImageActionAvailability } from "./image-action-availability";
import SpaceCanvasImageOverflowActions from "./SpaceCanvasImageOverflowActions.vue";
import SpaceCanvasImageRemoveObjectTrigger from "./SpaceCanvasImageRemoveObjectTrigger.vue";
const props = defineProps<{
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
/** Derivations the node bar already shows, so the menu does not repeat them. */
const BAR_DERIVATIONS = [
  { bar: "image-remove-background", action: "remove-image-background" },
  { bar: "image-split-layers", action: "split-image-layers" },
] as const satisfies readonly { bar: CanvasBarAction; action: CreationIntentId }[];
const barDerivations = computed<readonly CreationIntentId[]>(() =>
  BAR_DERIVATIONS.filter(({ bar }) => canvasBarPlacement(bar, props.minimal) === "bar").map(
    ({ action }) => action,
  ),
);
</script>
<template>
  <SpaceCanvasSelectionMenu
    :actions="actions"
    :derivations="derivations"
    node-bar
    node-tools-visible
    :exclude-actions="barDerivations"
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
