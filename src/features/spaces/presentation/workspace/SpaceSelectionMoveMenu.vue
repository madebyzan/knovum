<script setup lang="ts">
import { FolderInput } from "@lucide/vue";
import { sortBy } from "es-toolkit";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiDropdownMenu from "../../../../shared/ui/UiDropdownMenu.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useSpaceStore } from "../../session/space-store";

const emit = defineEmits<{ move: [spaceId: string] }>();
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
  <UiDropdownMenu
    side="top"
    align="center"
    :side-offset="7"
    :items="menuItems.length ? menuItems : undefined"
  >
    <template #trigger>
      <UiIconButton :label="t('spaces.navigation.moveToSpace')" size="sm">
        <FolderInput class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
      </UiIconButton>
    </template>
    <p v-if="!targets.length" class="px-2 py-2 text-caption text-foreground-tertiary">
      {{ t("spaces.navigation.noMoveTargets") }}
    </p>
  </UiDropdownMenu>
</template>
