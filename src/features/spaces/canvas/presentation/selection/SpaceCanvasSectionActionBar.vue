<script setup lang="ts">
import { ChevronsDownUp, ChevronsUpDown, Focus, Shrink } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { CreationIntentDecisions } from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasActionToolbar from "./SpaceCanvasActionToolbar.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasSectionLayoutControls from "./SpaceCanvasSectionLayoutControls.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  availableWidth: number;
}>();
const { t } = useI18n();
const state = computed(() => props.actions.state.value);
const section = computed(() => state.value.section);
const identity = computed(
  () =>
    `${section.value?.id}:${section.value?.collapsed}:${section.value?.autoLayout}:${state.value.content.length}`,
);
</script>

<template>
  <SpaceCanvasActionToolbar
    v-slot="{ compact, minimal }"
    :label="t('spaces.canvas.selection.label')"
    :available-width="availableWidth"
    :identity="identity"
  >
    <template v-if="section && !section.collapsed">
      <SpaceCanvasSectionLayoutControls
        :section="section"
        :disabled="!state.canEditSection"
        :compact="compact"
        @auto-layout="actions.setSectionAutoLayout"
        @fill="actions.setSectionFill"
        @resize="actions.resizeSection"
      />
      <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
    </template>
    <template v-if="!minimal">
      <SpaceCanvasActionTrigger
        :label="t('spaces.canvas.sections.fitToContents')"
        :icon="Shrink"
        :compact="compact"
        :disabled="!state.canFitSection"
        @invoke="actions.fitSection"
      />
      <SpaceCanvasActionTrigger
        :label="t(section?.collapsed ? 'spaces.canvas.sections.expand' : 'spaces.canvas.sections.collapse')"
        :icon="section?.collapsed ? ChevronsUpDown : ChevronsDownUp"
        :compact="compact"
        @invoke="actions.toggleSectionCollapsed"
      />
      <SpaceCanvasActionTrigger
        v-if="state.content.length"
        :label="t('spaces.scenes.focus.selection')"
        :icon="Focus"
        :compact="compact"
        @invoke="actions.focus"
      />
    </template>
    <SpaceCanvasSelectionMenu :actions="actions" :derivations="derivations" node-bar />
    <SpaceCanvasNodeAccessActions
      :export-target="actions.exportTarget.value"
      @inspect="actions.inspect"
      @save="actions.exportFormat"
    />
  </SpaceCanvasActionToolbar>
</template>
