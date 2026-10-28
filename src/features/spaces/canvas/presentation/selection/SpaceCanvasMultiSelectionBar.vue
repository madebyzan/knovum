<script setup lang="ts">
import {
  AlignStartVertical,
  ChevronDown,
  Group,
  LayoutGrid,
  Link2,
  Sparkles,
  SquareDashed,
} from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { computed, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type {
  CreationIntentDecisions,
  CreationIntentId,
} from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "../canvas-action-classes";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasAlignmentItems from "./SpaceCanvasAlignmentItems.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import SpaceSelectionMoveMenu from "./SpaceSelectionMoveMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useNodeActionBarDensity } from "./use-node-action-bar-density";

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
const { t, locale } = useI18n();
const state = computed(() => props.actions.state.value);
const bar = useTemplateRef<HTMLElement>("bar");
const identity = computed(() =>
  [
    locale.value,
    state.value.structureNodeIds.length,
    state.value.arrangement.align,
    state.value.arrangement.autoLayout,
    state.value.canCreateGroup,
    state.value.canCreateSection,
    state.value.canConnect,
    state.value.canAsk,
  ].join(":"),
);
const { compact, ready } = useNodeActionBarDensity(bar, toRef(props, "availableWidth"), identity);
const alignLabel = computed(() => t("spaces.canvas.align.title"));
</script>

<template>
  <div ref="bar" :class="ready ? 'visible' : 'invisible'" :style="{ maxWidth: `${availableWidth}px` }">
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      :loop="false"
      :aria-label="t('spaces.canvas.selection.label')"
    >
      <span
        v-if="state.structureNodeIds.length > 1"
        class="mr-0.5 ml-1.5 grid h-5 min-w-5 place-items-center rounded-full bg-surface-control px-1.5 text-caption font-medium text-fg-secondary tabular-nums"
        :aria-label="t('spaces.canvas.selection.count', { count: state.structureNodeIds.length })"
      >
        {{ state.structureNodeIds.length }}
      </span>
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
          <ToolbarButton as-child>
            <button
              type="button"
              :class="[CANVAS_ACTION_BUTTON_CLASSES, 'gap-0.5 px-2']"
              :aria-label="alignLabel"
              :title="alignLabel"
            >
              <AlignStartVertical class="size-4.5" :stroke-width="1.6" />
              <ChevronDown class="size-3 text-fg-tertiary" />
            </button>
          </ToolbarButton>
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
    </ToolbarRoot>
  </div>
</template>
