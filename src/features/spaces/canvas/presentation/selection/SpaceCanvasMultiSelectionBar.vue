<script setup lang="ts">
import { Group, LayoutGrid, Link2, SquareDashed } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { hasVisibleCreationActions } from "../../../knowledge-derivations/presentation/visible-derivation-actions";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasActionToolbar from "./SpaceCanvasActionToolbar.vue";
import SpaceCanvasAlignmentPalette from "./SpaceCanvasAlignmentPalette.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasSelectionAiActions from "./SpaceCanvasSelectionAiActions.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceSelectionMoveMenu from "./SpaceSelectionMoveMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

/**
 * Several roots at once. Ask & Create lead alongside spatial arrangement and containers; transfer
 * and overflow actions stay compact at the end.
 */
const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  availableWidth: number;
}>();
const emit = defineEmits<{ invokeDerivation: [actionId: CreationIntentId] }>();
const { t } = useI18n();
const state = computed(() => props.actions.state.value);
const hasCreationActions = computed(() => hasVisibleCreationActions(props.derivations));
const hasSpatialTools = computed(
  () =>
    state.value.arrangement.autoLayout ||
    state.value.arrangement.align ||
    state.value.canCreateGroup ||
    state.value.canCreateSection ||
    state.value.canConnect,
);
const identity = computed(() =>
  [
    state.value.structureNodeIds.length,
    state.value.arrangement.align,
    state.value.arrangement.autoLayout,
    state.value.canCreateGroup,
    state.value.canCreateSection,
    state.value.canConnect,
    state.value.canAsk,
    hasCreationActions.value,
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
    <SpaceCanvasSelectionAiActions
      :can-ask="state.canAsk"
      :derivations="derivations"
      :compact="compact"
      :separator="hasSpatialTools"
      @ask="actions.ask"
      @invoke-derivation="emit('invokeDerivation', $event)"
    />
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
    <SpaceSelectionMoveMenu
      v-if="state.structureNodeIds.length"
      :disabled="!actions.editorProtection.canTransfer.value"
      @move="actions.moveToSpace"
    />
    <SpaceCanvasSelectionMenu :actions="actions" arrange-on-bar />
    <SpaceCanvasNodeAccessActions
      v-if="actions.exportTarget.value"
      :export-target="actions.exportTarget.value"
      :show-info="false"
      @save="actions.exportFormat"
    />
  </SpaceCanvasActionToolbar>
</template>
