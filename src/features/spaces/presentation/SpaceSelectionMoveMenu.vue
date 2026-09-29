<script setup lang="ts">
import { ChevronRight, FolderInput } from "@lucide/vue";
import { sortBy } from "es-toolkit";
import { storeToRefs } from "pinia";
import { DropdownMenuPortal, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { SUBMENU_CONTENT_CLASSES } from "../../../shared/ui/ui-classes";
import UiDropdownMenu from "../../../shared/ui/UiDropdownMenu.vue";
import UiDropdownMenuItem from "../../../shared/ui/UiDropdownMenuItem.vue";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import { useSpaceStore } from "../session/space-store";

const emit = defineEmits<{ move: [spaceId: string] }>();
defineProps<{ submenu?: boolean }>();
const store = useSpaceStore();
const { activeSpaceId, spaces } = storeToRefs(store);
const { t } = useI18n();

const targets = computed(() =>
  sortBy(
    spaces.value.filter((space) => space.id !== activeSpaceId.value && !space.archivedAt),
    [(space) => -space.lastOpenedAt],
  ),
);

const menuItems = computed(() =>
  targets.value.map((space) => ({
    id: space.id,
    label: space.kind === "inbox" ? t("spaces.space.inbox") : space.title,
    onSelect: () => emit("move", space.id),
  })),
);
</script>

<template>
  <DropdownMenuSub v-if="submenu">
    <DropdownMenuSubTrigger
      class="flex h-interaction cursor-default items-center gap-2 rounded-md px-2 text-label text-fg-primary outline-none data-[highlighted]:bg-surface-control-hover"
    >
      <FolderInput class="size-3.5" />
      {{ t("spaces.space.moveTo") }}
      <ChevronRight class="ml-auto size-3 text-fg-tertiary" />
    </DropdownMenuSubTrigger>
    <DropdownMenuPortal>
      <DropdownMenuSubContent :class="SUBMENU_CONTENT_CLASSES">
        <UiDropdownMenuItem v-for="space in targets" :key="space.id" @select="emit('move', space.id)">
          {{ space.kind === "inbox" ? t("spaces.space.inbox") : space.title }}
        </UiDropdownMenuItem>
        <p v-if="!targets.length" class="px-2 py-2 text-caption text-fg-tertiary">
          {{ t("spaces.space.moveNoTargets") }}
        </p>
      </DropdownMenuSubContent>
    </DropdownMenuPortal>
  </DropdownMenuSub>
  <UiDropdownMenu
    v-else
    side="top"
    align="center"
    :side-offset="7"
    :items="menuItems.length ? menuItems : undefined"
  >
    <template #trigger>
      <UiIconButton :label="t('spaces.space.moveTo')" tooltip size="sm">
        <FolderInput class="size-3.5" :stroke-width="1.55" />
      </UiIconButton>
    </template>
    <p v-if="!targets.length" class="px-2 py-2 text-caption text-fg-tertiary">
      {{ t("spaces.space.moveNoTargets") }}
    </p>
  </UiDropdownMenu>
</template>
