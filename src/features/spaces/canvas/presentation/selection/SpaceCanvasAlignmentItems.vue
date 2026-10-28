<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { StructureArrangementAvailability } from "../../../structure/action-availability";
import type { CanvasAlignMode, CanvasDistributeMode } from "../../contracts/canvas-editor-port";

defineProps<{ availability: StructureArrangementAvailability }>();
const emit = defineEmits<{ align: [mode: CanvasAlignMode]; distribute: [mode: CanvasDistributeMode] }>();
const { t } = useI18n();
const ALIGN_MODES: readonly CanvasAlignMode[] = ["left", "center", "right", "top", "middle", "bottom"];
const DISTRIBUTE_MODES: readonly CanvasDistributeMode[] = ["horizontal", "vertical"];
</script>

<template>
  <UiDropdownMenuItem v-for="mode in ALIGN_MODES" :key="mode" @select="emit('align', mode)">
    {{ t(`spaces.canvas.align.${mode}`) }}
  </UiDropdownMenuItem>
  <template v-if="availability.distribute">
    <UiSeparator class="my-1" />
    <UiDropdownMenuItem v-for="mode in DISTRIBUTE_MODES" :key="mode" @select="emit('distribute', mode)">
      {{ t(`spaces.canvas.distribute.${mode}`) }}
    </UiDropdownMenuItem>
  </template>
</template>
