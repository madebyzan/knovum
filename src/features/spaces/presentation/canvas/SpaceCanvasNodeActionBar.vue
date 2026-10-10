<script setup lang="ts">
import { ToolbarRoot } from "reka-ui";
import { computed, nextTick, toRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import SpaceNodeTranscriptionAction from "../media/SpaceNodeTranscriptionAction.vue";
import { nodeSpeechOperationIds, type NodeSpeechOperation } from "../speech/node-speech-operations";
import SpaceNodeSpeechActions from "../speech/SpaceNodeSpeechActions.vue";
import SpaceNodeSpeechControls from "../speech/SpaceNodeSpeechControls.vue";
import { useNodeSeparation } from "../speech/use-node-separation";
import { CANVAS_ACTION_BAR_CLASSES } from "./canvas-action-classes";
import { canvasBarPlacement } from "./canvas-bar-density-policy";
import SpaceCanvasActionTrigger from "./SpaceCanvasActionTrigger.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceCanvasTextFormatBar from "./SpaceCanvasTextFormatBar.vue";
import SpaceCanvasVideoActions from "./SpaceCanvasVideoActions.vue";
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
watch(speechOperation, async (operation, previous) => {
  if (operation || !previous) return;
  await nextTick();
  bar.value?.querySelector<HTMLButtonElement>("button")?.focus({ preventScroll: true });
});
const { separation, toggleSeparation } = useNodeSeparation(toRef(props, "node"));
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
const { primaryLabel, primaryIcon, showPrimary, runPrimaryAction } = useNodePrimaryAction(
  toRef(props, "node"),
  props.actions,
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
  />
  <div v-else ref="bar" :class="ready ? 'visible' : 'invisible'" :style="{ maxWidth: `${availableWidth}px` }">
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      :loop="false"
      :aria-label="t('spaces.canvas.nodeActions.label')"
    >
      <SpaceCanvasActionTrigger
        v-if="showPrimary && canvasBarPlacement('node-primary', minimal) === 'bar'"
        :label="primaryLabel"
        :icon="primaryIcon"
        :compact="compact"
        class="font-medium text-fg-primary"
        @invoke="runPrimaryAction"
      />
      <SpaceCanvasTextFormatBar
        v-if="textKind"
        :kind="textKind"
        :compact="compact"
        v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
        @update="actions.setAppearance"
      />
      <SpaceCanvasVideoActions
        v-if="actions.selectedVideoNode.value && canvasBarPlacement('video-edit', minimal) === 'bar'"
        :compact="compact"
        :can-add-subtitles="derivations['add-video-subtitles'].state === 'ready'"
        @edit="actions.editVideo"
        @add-subtitles="emit('invokeDerivation', 'add-video-subtitles')"
      />
      <SpaceNodeSpeechActions
        v-if="node && speechOperations.length && canvasBarPlacement('node-speech', minimal) === 'bar'"
        :node="node"
        :compact="compact"
        :separating="Boolean(separation.activeRunId.value)"
        :separation-busy="separation.isBusy.value"
        @separate="toggleSeparation"
        @begin="speechOperation = $event"
      />
      <SpaceNodeTranscriptionAction
        v-if="node && isMedia && canvasBarPlacement('node-transcription', minimal) === 'bar'"
        :node="node"
        :compact="compact"
      />
      <SpaceCanvasSelectionMenu
        :actions="actions"
        :derivations="derivations"
        node-bar
        :node-tools-visible="
          canvasBarPlacement('node-primary', minimal) === 'menu' &&
          (showPrimary ||
            Boolean(speechOperations.length) ||
            isMedia ||
            Boolean(actions.selectedVideoNode.value))
        "
        @invoke-derivation="emit('invokeDerivation', $event)"
      >
        <template #node-tools>
          <SpaceCanvasActionTrigger
            v-if="showPrimary && canvasBarPlacement('node-primary', minimal) === 'menu'"
            :label="primaryLabel"
            :icon="primaryIcon"
            menu-item
            @invoke="runPrimaryAction"
          />
          <SpaceCanvasVideoActions
            v-if="actions.selectedVideoNode.value && canvasBarPlacement('video-edit', minimal) === 'menu'"
            menu-item
            :can-add-subtitles="derivations['add-video-subtitles'].state === 'ready'"
            @edit="actions.editVideo"
            @add-subtitles="emit('invokeDerivation', 'add-video-subtitles')"
          />
          <SpaceNodeSpeechActions
            v-if="node && speechOperations.length && canvasBarPlacement('node-speech', minimal) === 'menu'"
            :node="node"
            menu-item
            :separating="Boolean(separation.activeRunId.value)"
            :separation-busy="separation.isBusy.value"
            @separate="toggleSeparation"
            @begin="speechOperation = $event"
          />
          <SpaceNodeTranscriptionAction
            v-if="node && isMedia && canvasBarPlacement('node-transcription', minimal) === 'menu'"
            :node="node"
            menu-item
          />
        </template>
      </SpaceCanvasSelectionMenu>
      <UiActionHelp v-if="separation.error.value" :label="separation.error.value" />
      <SpaceCanvasNodeAccessActions @inspect="actions.inspect" @export="actions.exportSelection" />
    </ToolbarRoot>
  </div>
</template>
