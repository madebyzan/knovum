<script setup lang="ts">
import { Group, Info, Link2, Sparkles } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../knowledge-derivations/catalog/creation-intent-definitions";
import type { NodeSpeechOperation } from "../speech/node-speech-operations";
import { CANVAS_ACTION_BAR_CLASSES } from "./canvas-action-classes";
import SpaceCanvasImageActionBar from "./SpaceCanvasImageActionBar.vue";
import SpaceCanvasNodeActionBar from "./SpaceCanvasNodeActionBar.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceSelectionMoveMenu from "./SpaceSelectionMoveMenu.vue";
import { useCanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{
  derivations: CreationIntentDecisions;
  availableWidth: number;
  angleNodeId: string | undefined;
}>();
const emit = defineEmits<{
  invokeDerivation: [actionId: CreationIntentId];
  "update:angleNodeId": [nodeId: string | undefined];
}>();
const speechOperation = defineModel<NodeSpeechOperation | undefined>("speechOperation");
const actions = useCanvasSelectionActions();
const { t } = useI18n();
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
  <SpaceCanvasNodeActionBar
    v-else-if="singleNode || singleSection"
    :actions="actions"
    :derivations="derivations"
    v-model:speech-operation="speechOperation"
    :node="singleNode"
    :available-width="availableWidth"
    @invoke-derivation="emit('invokeDerivation', $event)"
  />
  <ToolbarRoot
    v-else-if="actions.visible.value"
    :class="CANVAS_ACTION_BAR_CLASSES"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.canvas.selection.label')"
  >
    <UiBadge v-if="actions.state.value.structureNodeIds.length > 1" size="sm" class="mx-1">
      {{ actions.state.value.structureNodeIds.length }}
    </UiBadge>
    <ToolbarButton as-child>
      <UiIconButton :label="t('spaces.inspector.title')" tooltip size="sm" @click="actions.inspect">
        <Info class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.canConnect" as-child>
      <UiIconButton :label="t('spaces.canvas.relations.connect')" tooltip size="sm" @click="actions.connect">
        <Link2 class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.canAsk" as-child>
      <UiIconButton :label="t('spaces.conversation.askSelection')" tooltip size="sm" @click="actions.ask">
        <Sparkles class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <SpaceSelectionMoveMenu v-if="actions.state.value.structureNodeIds.length" @move="actions.moveToSpace" />
    <ToolbarButton v-if="actions.state.value.canCreateGroup" as-child>
      <UiIconButton
        :label="t('spaces.canvas.groups.create')"
        shortcut="$mod+G"
        size="sm"
        @click="actions.createGroup"
      >
        <Group class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.canCreateSection" as-child>
      <UiIconButton
        :label="t('spaces.canvas.sections.create')"
        tooltip
        size="sm"
        @click="actions.createSection"
      >
        <Group class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
    <SpaceCanvasSelectionMenu
      :actions="actions"
      :derivations="derivations"
      @invoke-derivation="emit('invokeDerivation', $event)"
    />
  </ToolbarRoot>
</template>
