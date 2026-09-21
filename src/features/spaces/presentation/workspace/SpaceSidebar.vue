<script setup lang="ts">
import { Clock3, Inbox, LayoutGrid, Star, Trash2 } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../../session/space-store";
import { useSpaceManagementMenu } from "../../workspace/use-space-management-menu";
import SpaceSidebarOutline from "./SpaceSidebarOutline.vue";

const store = useSpaceStore();
const management = useSpaceManagementMenu();
const { spaces, activeSpaceId, workspaceMode, libraryScope } = storeToRefs(store);
const { t } = useI18n();
const rowClass =
  "flex h-(--knovum-compact-item-height) w-full items-center gap-2 rounded-control px-2 text-left text-label text-foreground-secondary hover:bg-(--knovum-sidebar-item-background-hover) hover:text-foreground data-[active=true]:bg-(--knovum-sidebar-item-background-selected) data-[active=true]:text-foreground";
const inbox = computed(() => spaces.value.find((space) => space.kind === "inbox"));
</script>

<template>
  <aside class="flex h-full min-h-0 flex-col select-none">
    <header class="flex h-(--knovum-control-height-lg) shrink-0 items-center px-3">
      <strong class="min-w-0 truncate text-body-lg font-semibold">{{ t("spaces.space.title") }}</strong>
    </header>
    <nav class="shrink-0 space-y-0.5 px-2.5" :aria-label="t('spaces.space.title')">
      <button
        v-if="inbox"
        type="button"
        :data-active="workspaceMode === 'space' && activeSpaceId === inbox.id"
        :class="rowClass"
        @click="store.openSpace(inbox.id)"
        @contextmenu.prevent="management.show(inbox)"
      >
        <Inbox class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
        {{ t("spaces.space.inbox") }}
      </button>
      <button
        type="button"
        :class="rowClass"
        :data-active="workspaceMode === 'library' && libraryScope === 'all'"
        @click="store.showSpaceLibrary('all')"
      >
        <LayoutGrid class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
        {{ t("spaces.space.allSpaces") }}
      </button>
      <button
        type="button"
        :class="rowClass"
        :data-active="workspaceMode === 'library' && libraryScope === 'recent'"
        @click="store.showSpaceLibrary('recent')"
      >
        <Clock3 class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
        {{ t("spaces.space.recentSpaces") }}
      </button>
      <button
        type="button"
        :class="rowClass"
        :data-active="workspaceMode === 'library' && libraryScope === 'favorites'"
        @click="store.showSpaceLibrary('favorites')"
      >
        <Star class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
        {{ t("spaces.space.favoriteSpaces") }}
      </button>
    </nav>
    <SpaceSidebarOutline v-if="workspaceMode === 'space' && store.activeSpace" />
    <div v-if="workspaceMode !== 'space' || !store.activeSpace" class="flex-1" />
    <footer class="shrink-0 px-2.5 py-2">
      <button type="button" :class="rowClass" @click="store.recentlyDeletedSpaceId = null">
        <Trash2 class="size-(--knovum-icon-size-sm)" :stroke-width="1.55" />
        {{ t("spaces.navigation.recentlyDeleted") }}
      </button>
    </footer>
  </aside>
</template>
