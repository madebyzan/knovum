<script setup lang="ts">
import { ChevronRight, CornerUpLeft, FolderInput } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../../shared/ui/UiDropdownMenuItem.vue";

defineProps<{
  rowClass: string;
  submenuClass: string;
  targets: readonly { id: string | null; title: string }[];
}>();
const emit = defineEmits<{ reparent: [parentId: string | null] }>();
const { t } = useI18n();
</script>

<template>
  <DropdownMenuSub v-if="targets.length">
    <DropdownMenuSubTrigger :class="rowClass">
      <FolderInput class="size-3.5" />
      <span class="min-w-0 flex-1">{{ t("spaces.canvas.moveToContainer") }}</span>
      <ChevronRight class="size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
        <UiDropdownMenuItem
          v-for="target in targets"
          :key="target.id ?? 'root'"
          @select="emit('reparent', target.id)"
        >
          <CornerUpLeft v-if="target.id === null" class="size-3.5" />
          <FolderInput v-else class="size-3.5" />
          <span class="truncate">{{ target.title }}</span>
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
