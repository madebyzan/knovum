<script setup lang="ts">
import { ChevronRight, CornerUpLeft, FolderInput } from "@lucide/vue";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiDropdownMenuItem from "../../../../shared/ui/UiDropdownMenuItem.vue";

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
      <FolderInput class="size-(--knovum-icon-size-sm)" />
      <span class="min-w-0 flex-1">{{ t("spaces.canvas.moveToContainer") }}</span>
      <ChevronRight class="size-(--knovum-icon-size-xs) text-foreground-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :side-offset="4" :align-offset="-4" :class="submenuClass">
        <UiDropdownMenuItem
          v-for="target in targets"
          :key="target.id ?? 'root'"
          @select="emit('reparent', target.id)"
        >
          <CornerUpLeft v-if="target.id === null" class="size-(--knovum-icon-size-sm)" />
          <FolderInput v-else class="size-(--knovum-icon-size-sm)" />
          <span class="truncate">{{ target.title }}</span>
        </UiDropdownMenuItem>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
</template>
