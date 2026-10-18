<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { computed, nextTick, shallowRef, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiActionHelp from "../../../../../shared/ui/UiActionHelp.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import {
  nodeSpeechOperationIds,
  type NodeSpeechOperation,
} from "../../../speech/presentation/node-speech-operations";
import SpaceNodeSeparationBar from "../../../speech/presentation/SpaceNodeSeparationBar.vue";
import SpaceNodeSpeechControls from "../../../speech/presentation/SpaceNodeSpeechControls.vue";
import { useNodeSeparation } from "../../../speech/presentation/use-node-separation";
import { CANVAS_ACTION_BAR_CLASSES } from "../canvas-action-classes";
import SpaceCanvasTextFormatBar from "../SpaceCanvasTextFormatBar.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasNodeTools from "./SpaceCanvasNodeTools.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useNodeActionBarDensity } from "./use-node-action-bar-density";
import { useNodePrimaryAction } from "./use-node-primary-action";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  node?: SpaceCanvasNode | undefined;
  availableWidth: number;
  availableHeight?: number | undefined;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t, locale } = useI18n();
const speechOperation = defineModel<NodeSpeechOperation | undefined>("speechOperation");
const focusOperation = shallowRef<NodeSpeechOperation>();
watch(speechOperation, (operation, previous) => {
  if (!operation && previous) focusOperation.value = previous;
});
const { separation, unavailableReason, startSeparation } = useNodeSeparation(toRef(props, "node"));
function beginSeparation() {
  speechOperation.value = "separate";
  if (!separation.activeRunId.value) void startSeparation();
}
const speechOperations = computed(() => (props.node ? nodeSpeechOperationIds(props.node) : []));
const bar = useTemplateRef<HTMLElement>("bar");
const densityIdentity = computed(
  () =>
    `${locale.value}:${props.node?.id}:${props.node?.data.kind}:${speechOperations.value.join(",")}:${props.actions.state.value.canOpen}:${Boolean(props.actions.selectedVideoNode.value)}:${Boolean(props.actions.state.value.section)}`,
);
const { compact, minimal, ready } = useNodeActionBarDensity(
  bar,
  toRef(props, "availableWidth"),
  densityIdentity,
);
const { primary, asksInstead } = useNodePrimaryAction(toRef(props, "node"), props.actions);
watch(
  [ready, focusOperation],
  async ([isReady, operation]) => {
    if (!isReady || !operation) return;
    await nextTick();
    const trigger =
      bar.value?.querySelector<HTMLButtonElement>(`button[data-operation="${operation}"]`) ??
      bar.value?.querySelector<HTMLButtonElement>('button[aria-haspopup="menu"]');
    trigger?.focus({ preventScroll: true });
    focusOperation.value = undefined;
  },
  { flush: "post" },
);
const isMedia = computed(() => props.node?.data.kind === "audio" || props.node?.data.kind === "video");
const textKind = computed(() => {
  const kind = props.node?.data.kind;
  return kind === "note" || kind === "text" ? kind : undefined;
});
</script>

<template>
  <SpaceNodeSpeechControls
    v-if="speechOperation"
    v-model:operation="speechOperation"
    :node="node"
    :available-width="availableWidth"
    :available-height="availableHeight"
  >
    <template #separation="{ close }">
      <SpaceNodeSeparationBar :separation="separation" @retry="startSeparation" @close="close" />
    </template>
  </SpaceNodeSpeechControls>
  <div v-else ref="bar" :class="ready ? 'visible' : 'invisible'" :style="{ maxWidth: `${availableWidth}px` }">
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      :loop="false"
      :aria-label="t('spaces.canvas.nodeActions.label')"
    >
      <SpaceCanvasNodeTools
        :minimal="minimal"
        :actions="actions"
        :derivations="derivations"
        :node="node"
        :compact="compact"
        :separating="Boolean(separation.activeRunId.value)"
        :separation-busy="separation.isBusy.value"
        :separation-unavailable-reason="unavailableReason"
        @separate="beginSeparation"
        @begin-speech="speechOperation = $event"
        @invoke-derivation="emit('invokeDerivation', $event)"
      >
        <SpaceCanvasTextFormatBar
          v-if="textKind"
          :kind="textKind"
          :compact="compact"
          v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
          @update="actions.setAppearance"
        />
      </SpaceCanvasNodeTools>
      <UiActionHelp v-if="separation.error.value" :label="separation.error.value" />
      <SpaceCanvasNodeAccessActions
        :export-target="actions.exportTarget.value"
        @inspect="actions.inspect"
        @save="actions.exportFormat"
      />
      <SpaceCanvasSelectionMenu
        :actions="actions"
        :derivations="derivations"
        node-bar
        :exclude-ask="asksInstead"
        :node-tools-visible="
          minimal &&
          (Boolean(primary) ||
            Boolean(speechOperations.length) ||
            isMedia ||
            Boolean(actions.selectedVideoNode.value))
        "
        @invoke-derivation="emit('invokeDerivation', $event)"
      >
        <template #node-tools>
          <SpaceCanvasNodeTools
            :actions="actions"
            :derivations="derivations"
            :node="node"
            menu-item
            :separating="Boolean(separation.activeRunId.value)"
            :separation-busy="separation.isBusy.value"
            :separation-unavailable-reason="unavailableReason"
            @separate="beginSeparation"
            @begin-speech="speechOperation = $event"
            @invoke-derivation="emit('invokeDerivation', $event)"
          />
        </template>
      </SpaceCanvasSelectionMenu>
    </ToolbarRoot>
  </div>
</template>
