<script setup lang="ts">
import { AlignHorizontalSpaceAround, ChevronRight, Columns3, Rows3 } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import UiSeparator from "../../../../../shared/ui/UiSeparator.vue";
import type { AutoLayoutFlow } from "../../../contracts/generated/AutoLayoutFlow";
import type { StructureArrangementAvailability } from "../../../structure/action-availability";
import type { CanvasAlignMode, CanvasDistributeMode } from "../../contracts/canvas-editor-port";
import SpaceCanvasAlignmentItems from "./SpaceCanvasAlignmentItems.vue";

defineProps<{ availability: StructureArrangementAvailability; rowClass: string; submenuClass: string }>();
const emit = defineEmits<{
  align: [mode: CanvasAlignMode];
  autoLayout: [flow: AutoLayoutFlow];
  distribute: [mode: CanvasDistributeMode];
}>();
const { t } = useI18n();
</script>

<template>
  <DropdownMenuSub v-if="availability.align">
    <DropdownMenuSubTrigger :class="rowClass">
      <AlignHorizontalSpaceAround class="size-3.5" />
      <span class="min-w-0 flex-1">{{ t("spaces.canvas.arrange.title") }}</span>
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
        <SpaceCanvasAlignmentItems
          :availability="availability"
          @align="emit('align', $event)"
          @distribute="emit('distribute', $event)"
        />
        <template v-if="availability.autoLayout">
          <UiSeparator class="my-1" />
          <UiDropdownMenuItem @select="emit('autoLayout', 'row')">
            <Columns3 class="size-3.5" />{{ t("spaces.canvas.arrange.autoHorizontal") }}
          </UiDropdownMenuItem>
          <UiDropdownMenuItem @select="emit('autoLayout', 'column')">
            <Rows3 class="size-3.5" />{{ t("spaces.canvas.arrange.autoVertical") }}
          </UiDropdownMenuItem>
        </template>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
