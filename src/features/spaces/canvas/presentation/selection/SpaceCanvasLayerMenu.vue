<script setup lang="ts">
import { BringToFront, ChevronRight, Layers3, SendToBack } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";
import type { CanvasLayerMove } from "../../contracts/canvas-editor-port";

defineProps<{ rowClass: string; submenuClass: string }>();
const emit = defineEmits<{ move: [mode: CanvasLayerMove] }>();
const { t } = useI18n();
const LAYER_MOVES: readonly { mode: CanvasLayerMove; label: string; icon: typeof BringToFront }[] = [
  { mode: "front", label: "spaces.canvas.arrange.toFront", icon: BringToFront },
  { mode: "forward", label: "spaces.canvas.arrange.forward", icon: BringToFront },
  { mode: "backward", label: "spaces.canvas.arrange.backward", icon: SendToBack },
  { mode: "back", label: "spaces.canvas.arrange.toBack", icon: SendToBack },
];
</script>

<template>
  <DropdownMenuSub>
    <DropdownMenuSubTrigger :class="rowClass">
      <Layers3 class="size-3.5" />
      <span class="min-w-0 flex-1">{{ t("spaces.canvas.layerOrder") }}</span>
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
        <UiDropdownMenuItem v-for="move in LAYER_MOVES" :key="move.mode" @select="emit('move', move.mode)">
          <component :is="move.icon" class="size-3.5" />{{ t(move.label) }}
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
