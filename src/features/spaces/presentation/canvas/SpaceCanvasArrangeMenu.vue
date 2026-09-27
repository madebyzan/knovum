<script setup lang="ts">
import { AlignHorizontalSpaceAround, ChevronRight, Workflow } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";
import UiSeparator from "../../../../shared/ui/UiSeparator.vue";
import type {
  CanvasAlignMode,
  CanvasAutoLayoutDirection,
  CanvasDistributeMode,
} from "../../canvas/contracts/canvas-editor-port";

defineProps<{ count: number; rowClass: string; submenuClass: string }>();
const emit = defineEmits<{
  align: [mode: CanvasAlignMode];
  autoLayout: [direction: CanvasAutoLayoutDirection];
  distribute: [mode: CanvasDistributeMode];
}>();
const { t } = useI18n();
const alignModes: readonly CanvasAlignMode[] = ["left", "center", "right", "top", "middle", "bottom"];
const distributeModes: readonly CanvasDistributeMode[] = ["horizontal", "vertical"];
</script>

<template>
  <DropdownMenuSub v-if="count >= 2">
    <DropdownMenuSubTrigger :class="rowClass">
      <AlignHorizontalSpaceAround class="size-3.5" />
      <span class="min-w-0 flex-1">{{ t("spaces.canvas.arrange.title") }}</span>
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
        <UiDropdownMenuItem v-for="mode in alignModes" :key="mode" @select="emit('align', mode)">
          {{ t(`spaces.canvas.align.${mode}`) }}
        </UiDropdownMenuItem>
        <template v-if="count >= 3">
          <UiSeparator class="my-1" />
          <UiDropdownMenuItem v-for="mode in distributeModes" :key="mode" @select="emit('distribute', mode)">
            {{ t(`spaces.canvas.distribute.${mode}`) }}
          </UiDropdownMenuItem>
        </template>
        <UiSeparator class="my-1" />
        <UiDropdownMenuItem @select="emit('autoLayout', 'RIGHT')">
          <Workflow class="size-3.5" />{{ t("spaces.canvas.arrange.autoHorizontal") }}
        </UiDropdownMenuItem>
        <UiDropdownMenuItem @select="emit('autoLayout', 'DOWN')">
          <Workflow class="size-3.5 rotate-90" />{{ t("spaces.canvas.arrange.autoVertical") }}
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
