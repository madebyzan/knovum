<script setup lang="ts">
import { computed, toRef } from "vue";

import type { LocalModelId } from "../../../../../shared/contracts/generated/LocalModelId";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { addVideoSubtitles } from "../../../knowledge-derivations/subtitled-video";
import SpaceNodeTranscriptionAction from "../../../media/presentation/SpaceNodeTranscriptionAction.vue";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceCredentialPayload } from "../../../model/nodes/node-payload";
import {
  nodeSpeechOperationIds,
  type NodeSpeechOperation,
} from "../../../speech/presentation/node-speech-operations";
import SpaceNodeSpeechActions from "../../../speech/presentation/SpaceNodeSpeechActions.vue";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasVideoActions from "../SpaceCanvasVideoActions.vue";
import SpaceCanvasCredentialActions from "./SpaceCanvasCredentialActions.vue";
import SpaceCanvasSelectionAiActions from "./SpaceCanvasSelectionAiActions.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useNodePrimaryAction } from "./use-node-primary-action";

/**
 * Node-specific tools split by frequency: primary actions (Ask/Create, Preview/Open, Format, Video
 * Edit, Subtitles, Credential Copy) sit on the toolbar and overflow when minimal; lower-frequency
 * media and speech operations (transcribe, dubbing, audiobook, voice clone, separation) live in the
 * overflow menu.
 */
const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node: SpaceCanvasNode | undefined;
  compact?: boolean;
  minimal?: boolean;
  menuItem?: boolean;
  includePrimary?: boolean;
  separating: boolean;
  separationBusy: boolean;
  separationUnavailableReason: string | undefined;
}>();
const emit = defineEmits<{
  invokeDerivation: [actionId: CreationIntentId];
  beginSpeech: [operation: NodeSpeechOperation];
  separate: [model?: LocalModelId];
}>();
const { primary } = useNodePrimaryAction(toRef(props, "node"), props.actions);
const speechOperations = computed(() => (props.node ? nodeSpeechOperationIds(props.node) : []));
const isMedia = computed(() => props.node?.data.kind === "audio" || props.node?.data.kind === "video");
const isText = computed(() => props.node?.data.kind === "note" || props.node?.data.kind === "text");
const credentialNode = computed(() =>
  props.node?.data.kind === "credential"
    ? (props.node as SpaceCanvasNode<SpaceCredentialPayload>)
    : undefined,
);
const showPrimary = computed(() => (props.menuItem ? Boolean(props.includePrimary) : !props.minimal));
const hasNodeTools = computed(
  () =>
    Boolean(primary.value) ||
    Boolean(credentialNode.value) ||
    isText.value ||
    Boolean(props.actions.selectedVideoNode.value),
);
</script>

<template>
  <template v-if="showPrimary">
    <SpaceCanvasSelectionAiActions
      :can-ask="actions.state.value.canAsk"
      :derivations="derivations"
      :compact="compact"
      :menu-item="menuItem"
      :separator="hasNodeTools"
      @ask="actions.ask"
      @invoke-derivation="emit('invokeDerivation', $event)"
    />
    <SpaceCanvasCredentialActions
      v-if="credentialNode"
      :node="credentialNode"
      :compact="compact"
      :menu-item="menuItem"
    />
    <SpaceCanvasActionTrigger
      v-if="primary"
      :label="primary.label"
      :icon="primary.icon"
      :compact="compact"
      :menu-item="menuItem"
      @invoke="primary.run"
    />
    <slot />
    <SpaceCanvasVideoActions
      v-if="actions.selectedVideoNode.value"
      :compact="compact"
      :menu-item="menuItem"
      :can-add-subtitles="derivations['add-video-subtitles'].state === 'ready'"
      @edit="actions.editVideo"
      @add-subtitles="
        (model) =>
          model && node
            ? void addVideoSubtitles(node, model)
            : emit('invokeDerivation', 'add-video-subtitles')
      "
    />
  </template>
  <SpaceNodeTranscriptionAction v-if="menuItem && node && isMedia" :node="node" menu-item />
  <SpaceNodeSpeechActions
    v-if="menuItem && node && speechOperations.length"
    :node="node"
    menu-item
    :separating="separating"
    :separation-busy="separationBusy"
    :separation-unavailable-reason="separationUnavailableReason"
    @separate="emit('separate', $event)"
    @begin="emit('beginSpeech', $event)"
  />
</template>
