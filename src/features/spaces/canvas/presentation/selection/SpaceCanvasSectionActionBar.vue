<script setup lang="ts">
import { ChevronDown, ChevronRight, Focus, Scan } from "@lucide/vue";
import { ToolbarRoot } from "reka-ui";
import { computed, toRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import type { CreationIntentDecisions } from "../../../knowledge-derivations/catalog/creation-intent-definitions";
import { CANVAS_ACTION_BAR_CLASSES } from "../canvas-action-classes";
import SpaceCanvasActionTrigger from "../SpaceCanvasActionTrigger.vue";
import SpaceCanvasNodeAccessActions from "./SpaceCanvasNodeAccessActions.vue";
import SpaceCanvasSelectionMenu from "./SpaceCanvasSelectionMenu.vue";
import type { useCanvasSelectionActions } from "./use-canvas-selection-actions";
import { useNodeActionBarDensity } from "./use-node-action-bar-density";

const props = defineProps<{
  actions: ReturnType<typeof useCanvasSelectionActions>;
  derivations: CreationIntentDecisions;
  availableWidth: number;
}>();
const { t, locale } = useI18n();
const bar = useTemplateRef<HTMLElement>("bar");
const identity = computed(
  () =>
    `${locale.value}:${props.actions.state.value.section?.id}:${props.actions.state.value.section?.collapsed}:${props.actions.state.value.content.length}`,
);
const { compact, minimal, ready } = useNodeActionBarDensity(bar, toRef(props, "availableWidth"), identity);
</script>

<template>
  <div ref="bar" :class="ready ? 'visible' : 'invisible'" :style="{ maxWidth: `${availableWidth}px` }">
    <ToolbarRoot
      :class="CANVAS_ACTION_BAR_CLASSES"
      :loop="false"
      :aria-label="t('spaces.canvas.selection.label')"
    >
      <template v-if="!minimal">
        <SpaceCanvasActionTrigger
          :label="
            t(
              actions.state.value.section?.collapsed
                ? 'spaces.canvas.sections.expand'
                : 'spaces.canvas.sections.collapse',
            )
          "
          :icon="actions.state.value.section?.collapsed ? ChevronRight : ChevronDown"
          :compact="compact"
          @invoke="actions.toggleSectionCollapsed"
        />
        <SpaceCanvasActionTrigger
          :label="t('spaces.canvas.sections.fitToContents')"
          :icon="Scan"
          :compact="compact"
          :disabled="!actions.state.value.canFitSection"
          @invoke="actions.fitSection"
        />
        <SpaceCanvasActionTrigger
          v-if="actions.state.value.content.length"
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
    </ToolbarRoot>
  </div>
</template>
