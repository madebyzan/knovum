<script setup lang="ts">
import { computed, toRef } from "vue";

import { shouldShowAction } from "../../../../shared/contracts/action-availability";
import {
  DOCUMENT_CONVERSION_ACTION_IDS,
  type CreationIntentDecisions,
  type CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceDocumentConversionMenu from "../derivations/SpaceDocumentConversionMenu.vue";
import SpaceNodeTranscriptionAction from "../media/SpaceNodeTranscriptionAction.vue";
import { nodeSpeechOperationIds, type NodeSpeechOperation } from "../speech/node-speech-operations";
import SpaceNodeSpeechActions from "../speech/SpaceNodeSpeechActions.vue";
import SpaceCanvasActionTrigger from "./SpaceCanvasActionTrigger.vue";
import SpaceCanvasVideoActions from "./SpaceCanvasVideoActions.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useNodePrimaryAction } from "./use-node-primary-action";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node: SpaceCanvasNode | undefined;
  compact?: boolean;
  minimal?: boolean;
  menuItem?: boolean;
  separating: boolean;
  separationBusy: boolean;
  separationUnavailableReason: string | undefined;
}>();
const emit = defineEmits<{
  invokeDerivation: [actionId: CreationIntentId];
  beginSpeech: [operation: NodeSpeechOperation];
  separate: [];
}>();
const { primaryLabel, primaryIcon, showPrimary, runPrimaryAction } = useNodePrimaryAction(
  toRef(props, "node"),
  props.actions,
);
const speechOperations = computed(() => (props.node ? nodeSpeechOperationIds(props.node) : []));
const isMedia = computed(() => props.node?.data.kind === "audio" || props.node?.data.kind === "video");
const canConvert = computed(() =>
  DOCUMENT_CONVERSION_ACTION_IDS.some((id) => shouldShowAction(props.derivations[id])),
);
</script>

<template>
  <SpaceCanvasActionTrigger
    v-if="showPrimary && !minimal"
    :label="primaryLabel"
    :icon="primaryIcon"
    :compact="compact"
    :menu-item="menuItem"
    class="font-medium text-fg-primary"
    @invoke="runPrimaryAction"
  />
  <slot />
  <SpaceDocumentConversionMenu
    v-if="!minimal && canConvert"
    :decisions="derivations"
    :is-compact="compact"
    :is-submenu="menuItem"
    @invoke="emit('invokeDerivation', $event)"
  />
  <SpaceCanvasVideoActions
    v-if="!minimal && actions.selectedVideoNode.value"
    :compact="compact"
    :menu-item="menuItem"
    :can-add-subtitles="derivations['add-video-subtitles'].state === 'ready'"
    @edit="actions.editVideo"
    @add-subtitles="emit('invokeDerivation', 'add-video-subtitles')"
  />
  <SpaceNodeSpeechActions
    v-if="!minimal && node && speechOperations.length"
    :node="node"
    :compact="compact"
    :menu-item="menuItem"
    :separating="separating"
    :separation-busy="separationBusy"
    :separation-unavailable-reason="separationUnavailableReason"
    @separate="emit('separate')"
    @begin="emit('beginSpeech', $event)"
  />
  <SpaceNodeTranscriptionAction
    v-if="!minimal && node && isMedia"
    :node="node"
    :compact="compact"
    :menu-item="menuItem"
  />
</template>
