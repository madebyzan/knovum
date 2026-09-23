<script setup lang="ts">
import { Copy, Eye, ExternalLink, Group, Info, Link2, Scissors, Sparkles } from "@lucide/vue";
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
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceCanvasTextPayload } from "../../model/nodes/capabilities";
import SpaceSelectionMoveMenu from "../SpaceSelectionMoveMenu.vue";
import SpaceCanvasImageTools from "./SpaceCanvasImageTools.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceCanvasTextFormatBar from "./SpaceCanvasTextFormatBar.vue";
import { useCanvasSelectionActions } from "./use-canvas-selection-actions";

defineProps<{ derivations: CreationIntentDecisions }>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const actions = useCanvasSelectionActions();
const { t } = useI18n();
const selectedTextNode = computed(() => {
  if (actions.selectedNodes.value.length !== 1) return undefined;
  const node = actions.selectedNodes.value[0];
  return node && isCanvasTextNode(node) ? node : undefined;
});

function isCanvasTextNode(node: SpaceCanvasNode): node is SpaceCanvasNode<SpaceCanvasTextPayload> {
  return node.data.kind === "note" || node.data.kind === "text";
}
</script>

<template>
  <ToolbarRoot
    v-if="actions.visible.value"
    class="flex h-7 cursor-default items-center gap-0.5 rounded-control border border-border bg-overlay select-none"
    orientation="horizontal"
    :loop="false"
    :aria-label="t('spaces.navigation.selection')"
  >
    <UiBadge v-if="actions.state.value.structureNodeIds.length > 1" size="sm" class="mx-1">
      {{ actions.state.value.structureNodeIds.length }}
    </UiBadge>
    <SpaceCanvasTextFormatBar
      v-if="selectedTextNode"
      :kind="selectedTextNode.data.kind"
      v-bind="actions.selectedAppearance.value ? { appearance: actions.selectedAppearance.value } : {}"
      @update="actions.setAppearance"
    />
    <SpaceCanvasImageTools v-if="actions.selectedImageNode.value" :actions="actions" />
    <ToolbarButton v-if="actions.selectedVideoNode.value" as-child>
      <UiIconButton :label="t('spaces.media.editVideo')" size="sm" @click="actions.editVideo">
        <Scissors class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.canOpen" as-child>
      <UiIconButton
        :label="t(actions.state.value.opensSystemPreview ? 'spaces.transfer.systemPreview' : 'common.open')"
        size="sm"
        @click="actions.open"
      >
        <Eye
          v-if="actions.state.value.opensSystemPreview"
          class="size-(--knovum-icon-size-sm)"
          :stroke-width="1.55"
        />
        <ExternalLink v-else class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton as-child>
      <UiIconButton :label="t('spaces.navigation.inspect')" size="sm" @click="actions.inspect">
        <Info class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.canConnect" as-child>
      <UiIconButton :label="t('spaces.canvas.connect')" size="sm" @click="actions.connect">
        <Link2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.content.length" as-child>
      <UiIconButton :label="t('spaces.assistant.askSelection')" size="sm" @click="actions.ask">
        <Sparkles class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.structureNodeIds.length" as-child>
      <UiIconButton
        :label="t('spaces.navigation.duplicateSelection')"
        shortcut="$mod+D"
        size="sm"
        @click="actions.duplicate"
      >
        <Copy class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <SpaceSelectionMoveMenu v-if="actions.state.value.structureNodeIds.length" @move="actions.moveToSpace" />
    <ToolbarButton v-if="actions.state.value.canCreateGroup" as-child>
      <UiIconButton
        :label="t('spaces.canvas.groupSelection')"
        shortcut="$mod+G"
        size="sm"
        @click="actions.createGroup"
      >
        <Group class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </ToolbarButton>
    <ToolbarButton v-if="actions.state.value.canCreateSection" as-child>
      <UiIconButton :label="t('spaces.canvas.createSection')" size="sm" @click="actions.createSection">
        <Group class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
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
