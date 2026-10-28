<script setup lang="ts">
import { Group, LayoutGrid, Link2, Sparkles, SquareDashed } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasActionToolbar from "./SpaceCanvasActionToolbar.vue";
import SpaceCanvasAlignmentPalette from "./SpaceCanvasAlignmentPalette.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceSelectionMoveMenu from "./SpaceSelectionMoveMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

/**
 * Several roots at once. Arranging leads, as it is what a multi-selection is mostly for; then the
 * containers the roots can become; knowledge and transfer actions stay as icons.
 */
const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  availableWidth: number;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const state = computed(() => props.actions.state.value);
const identity = computed(() =>
  [
    state.value.structureNodeIds.length,
    state.value.arrangement.align,
    state.value.arrangement.autoLayout,
    state.value.canCreateGroup,
    state.value.canCreateSection,
    state.value.canConnect,
    state.value.canAsk,
  ].join(":"),
);
</script>

<template>
  <SpaceCanvasActionToolbar
    v-slot="{ compact }"
    :label="t('spaces.canvas.selection.label')"
    :available-width="availableWidth"
    :identity="identity"
  >
    <SpaceCanvasActionTrigger
      v-if="state.arrangement.autoLayout"
      :label="t('spaces.canvas.arrange.tidy')"
      :icon="LayoutGrid"
      :compact="compact"
      @invoke="actions.autoLayout('grid')"
    />
    <SpaceCanvasAlignmentPalette
      v-if="state.arrangement.align"
      :availability="state.arrangement"
      @align="actions.align"
      @distribute="actions.distribute"
    />
    <UiSeparator
      v-if="state.arrangement.align && (state.canCreateGroup || state.canCreateSection)"
      orientation="vertical"
      class="mx-0.5 h-4"
    />
    <SpaceCanvasActionTrigger
      v-if="state.canCreateGroup"
      :label="t('spaces.canvas.groups.create')"
      :icon="Group"
      :compact="compact"
      @invoke="actions.createGroup"
    />
    <SpaceCanvasActionTrigger
      v-if="state.canCreateSection"
      :label="t('spaces.canvas.sections.create')"
      :icon="SquareDashed"
      :compact="compact"
      @invoke="actions.createSection"
    />
    <SpaceCanvasActionTrigger
      v-if="state.canConnect"
      :label="t('spaces.canvas.relations.connect')"
      :icon="Link2"
      compact
      @invoke="actions.connect"
    />
    <SpaceCanvasActionTrigger
      v-if="state.canAsk"
      :label="t('spaces.conversation.askSelection')"
      :icon="Sparkles"
      compact
      @invoke="actions.ask"
    />
    <SpaceSelectionMoveMenu
      v-if="state.structureNodeIds.length"
      :disabled="!actions.editorProtection.canTransfer.value"
      @move="actions.moveToSpace"
    />
    <SpaceCanvasSelectionMenu
      :actions="actions"
      :derivations="derivations"
      node-bar
      @invoke-derivation="emit('invokeDerivation', $event)"
    />
    <SpaceCanvasNodeAccessActions
      :export-target="actions.exportTarget.value"
      @inspect="actions.inspect"
      @save="actions.exportFormat"
    />
  </SpaceCanvasActionToolbar>
</template>
