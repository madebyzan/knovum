<script setup lang="ts">
import { AlignStartVertical } from "@lucide/vue";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../../shared/ui/UiPopover.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { StructureArrangementAvailability } from "../../../structure/action-availability";
import type { CanvasAlignMode, CanvasDistributeMode } from "../../contracts/canvas-editor-port";
import SpaceCanvasMenuTrigger from "../SpaceCanvasMenuTrigger.vue";
import { CANVAS_ALIGN_ACTIONS, CANVAS_DISTRIBUTE_ACTIONS } from "./canvas-alignment-actions";

/**
 * Alignment as a design tool lays it out: one row of edge icons above the bar. It stays open, so a
 * selection can be aligned on both axes in one visit.
 */
defineProps<{ availability: StructureArrangementAvailability }>();
const emit = defineEmits<{ align: [mode: CanvasAlignMode]; distribute: [mode: CanvasDistributeMode] }>();
const { t } = useI18n();
</script>

<template>
  <UiPopover side="top" :side-offset="6" class="p-1">
    <template #trigger>
      <SpaceCanvasMenuTrigger :label="t('spaces.canvas.align.title')">
        <AlignStartVertical class="size-4.5" :stroke-width="1.6" />
      </SpaceCanvasMenuTrigger>
    </template>
    <ToolbarRoot class="flex items-center gap-0.5" :aria-label="t('spaces.canvas.align.title')">
      <ToolbarButton v-for="action in CANVAS_ALIGN_ACTIONS" :key="action.mode" as-child>
        <UiIconButton
          tooltip
          :label="t(`spaces.canvas.align.${action.mode}`)"
          @click="emit('align', action.mode)"
        >
          <component :is="action.icon" class="size-4" :stroke-width="1.6" />
        </UiIconButton>
      </ToolbarButton>
      <template v-if="availability.distribute">
        <UiSeparator orientation="vertical" class="mx-0.5 h-4" />
        <ToolbarButton v-for="action in CANVAS_DISTRIBUTE_ACTIONS" :key="action.mode" as-child>
          <UiIconButton
            tooltip
            :label="t(`spaces.canvas.distribute.${action.mode}`)"
            @click="emit('distribute', action.mode)"
          >
            <component :is="action.icon" class="size-4" :stroke-width="1.6" />
          </UiIconButton>
        </ToolbarButton>
      </template>
    </ToolbarRoot>
  </UiPopover>
</template>
