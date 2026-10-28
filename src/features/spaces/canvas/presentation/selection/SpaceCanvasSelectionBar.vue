<script setup lang="ts">
import { computed } from "vue";

import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import type { NodeSpeechOperation } from "../../../speech/presentation/node-speech-operations";
import SpaceCanvasImageActionBar from "../images/SpaceCanvasImageActionBar.vue";
import SpaceCanvasMultiSelectionBar from "./SpaceCanvasMultiSelectionBar.vue";
import SpaceCanvasNodeActionBar from "./SpaceCanvasNodeActionBar.vue";
import SpaceCanvasSectionActionBar from "./SpaceCanvasSectionActionBar.vue";
import { useCanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{
  derivations: CreationIntentDecisions;
  availableWidth: number;
  availableHeight?: number | undefined;
  angleNodeId: string | undefined;
}>();
const emit = defineEmits<{
  invokeDerivation: [actionId: CreationIntentId];
  "update:angleNodeId": [nodeId: string | undefined];
}>();
const speechOperation = defineModel<NodeSpeechOperation | undefined>("speechOperation");
const actions = useCanvasSelectionActions();
const singleNode = computed(() => {
  if (actions.state.value.structureNodeIds.length !== 1) return undefined;
  return actions.selectedNodes.value.length === 1 ? actions.selectedNodes.value[0] : undefined;
});
const singleSection = computed(
  () => actions.state.value.structureNodeIds.length === 1 && Boolean(actions.state.value.section),
);
</script>

<template>
  <SpaceCanvasImageActionBar
    v-if="actions.selectedImageNode.value && singleNode?.id === actions.selectedImageNode.value.id"
    :actions="actions"
    :derivations="derivations"
    :node="actions.selectedImageNode.value"
    :available-width="availableWidth"
    :angle-open="angleNodeId === actions.selectedImageNode.value.id"
    @update:angle-open="emit('update:angleNodeId', $event ? singleNode?.id : undefined)"
    @invoke-derivation="emit('invokeDerivation', $event)"
  />
  <SpaceCanvasSectionActionBar
    v-else-if="singleSection"
    :actions="actions"
    :derivations="derivations"
    :available-width="availableWidth"
  />
  <SpaceCanvasNodeActionBar
    v-else-if="singleNode"
    :actions="actions"
    :derivations="derivations"
    v-model:speech-operation="speechOperation"
    :node="singleNode"
    :available-width="availableWidth"
    :available-height="availableHeight"
    @invoke-derivation="emit('invokeDerivation', $event)"
  />
  <SpaceCanvasMultiSelectionBar
    v-else-if="actions.visible.value"
    :actions="actions"
    :derivations="derivations"
    :available-width="availableWidth"
    @invoke-derivation="emit('invokeDerivation', $event)"
  />
</template>
