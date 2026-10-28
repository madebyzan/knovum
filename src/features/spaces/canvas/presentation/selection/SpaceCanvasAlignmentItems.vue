<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { StructureArrangementAvailability } from "../../../structure/action-availability";
import type { CanvasAlignMode, CanvasDistributeMode } from "../../contracts/canvas-editor-port";
import { CANVAS_ALIGN_ACTIONS, CANVAS_DISTRIBUTE_ACTIONS } from "./canvas-alignment-actions";

defineProps<{ availability: StructureArrangementAvailability }>();
const emit = defineEmits<{ align: [mode: CanvasAlignMode]; distribute: [mode: CanvasDistributeMode] }>();
const { t } = useI18n();
</script>

<template>
  <UiDropdownMenuItem
    v-for="action in CANVAS_ALIGN_ACTIONS"
    :key="action.mode"
    @select="emit('align', action.mode)"
  >
    <component :is="action.icon" class="size-3.5" />{{ t(`spaces.canvas.align.${action.mode}`) }}
  </UiDropdownMenuItem>
  <template v-if="availability.distribute">
    <UiSeparator class="my-1" />
    <UiDropdownMenuItem
      v-for="action in CANVAS_DISTRIBUTE_ACTIONS"
      :key="action.mode"
      @select="emit('distribute', action.mode)"
    >
      <component :is="action.icon" class="size-3.5" />{{ t(`spaces.canvas.distribute.${action.mode}`) }}
    </UiDropdownMenuItem>
  </template>
</template>
