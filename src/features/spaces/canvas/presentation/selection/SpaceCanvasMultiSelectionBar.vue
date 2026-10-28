<script setup lang="ts">
import { AlignStartVertical, Group, LayoutGrid, Link2, Sparkles, SquareDashed } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiBadge from "../../../../../shared/ui/UiBadge.vue";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasMenuTrigger from "../SpaceCanvasMenuTrigger.vue";
import SpaceCanvasActionToolbar from "./SpaceCanvasActionToolbar.vue";
import SpaceCanvasAlignmentItems from "./SpaceCanvasAlignmentItems.vue";
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
const count = computed(() => state.value.structureNodeIds.length);
const countLabel = computed(() => t("spaces.canvas.selection.count", { count: count.value }));
const identity = computed(() =>
  [
    count.value,
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
    <UiBadge v-if="count > 1" class="mx-1.5 min-w-5 rounded-full px-1.5" :title="countLabel">
      <span aria-hidden="true">{{ count }}</span>
      <span class="sr-only">{{ countLabel }}</span>
    </UiBadge>
    <SpaceCanvasActionTrigger
      v-if="state.arrangement.autoLayout"
      :label="t('spaces.canvas.arrange.tidy')"
      :icon="LayoutGrid"
      :compact="compact"
      @invoke="actions.autoLayout('grid')"
    />
    <UiDropdownMenu
      v-if="state.arrangement.align"
      side="top"
      align="center"
      :side-offset="6"
      class="min-w-44"
    >
      <template #trigger>
        <SpaceCanvasMenuTrigger :label="t('spaces.canvas.align.title')">
          <AlignStartVertical class="size-4.5" :stroke-width="1.6" />
        </SpaceCanvasMenuTrigger>
      </template>
      <SpaceCanvasAlignmentItems
        :availability="state.arrangement"
        @align="actions.align"
        @distribute="actions.distribute"
      />
    </UiDropdownMenu>
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
      v-if="count"
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
