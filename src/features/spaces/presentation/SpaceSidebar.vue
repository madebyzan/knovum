<script setup lang="ts">
import { Clock3, Inbox, LayoutGrid, Star, Trash2 } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../session/space-store";
import { useSpaceManagementMenu } from "../workspace/use-space-management-menu";
import SpaceSidebarOutline from "./outline/SpaceSidebarOutline.vue";

const store = useSpaceStore();
const management = useSpaceManagementMenu();
const { spaces, activeSpaceId, workspaceMode, libraryScope } = storeToRefs(store);
const { t } = useI18n();
const rowClass =
  "flex h-interaction w-full items-center gap-2 rounded-md px-2 text-left text-label text-fg-secondary hover:bg-(--knovum-sidebar-item-bg-hover) hover:text-fg-primary data-[active=true]:bg-(--knovum-sidebar-item-bg-selected) data-[active=true]:text-fg-primary";
const inbox = computed(() => spaces.value.find((space) => space.kind === "inbox"));
</script>

<template>
  <aside class="flex h-full min-h-0 flex-col select-none">
    <header class="flex h-(--knovum-control-height-lg) shrink-0 items-center px-3">
      <strong class="min-w-0 truncate text-title-sm font-semibold">{{ t("spaces.space.title") }}</strong>
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
        <Inbox class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.inbox") }}
      </button>
      <button
        type="button"
        :class="rowClass"
        :data-active="workspaceMode === 'library' && libraryScope === 'all'"
        @click="store.showSpaceLibrary('all')"
      >
        <LayoutGrid class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.sections.all") }}
      </button>
      <button
        type="button"
        :class="rowClass"
        :data-active="workspaceMode === 'library' && libraryScope === 'recent'"
        @click="store.showSpaceLibrary('recent')"
      >
        <Clock3 class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.sections.recent") }}
      </button>
      <button
        type="button"
        :class="rowClass"
        :data-active="workspaceMode === 'library' && libraryScope === 'favorites'"
        @click="store.showSpaceLibrary('favorites')"
      >
        <Star class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.sections.favorites") }}
      </button>
    </nav>
    <SpaceSidebarOutline v-if="workspaceMode === 'space' && store.activeSpace" />
    <div v-if="workspaceMode !== 'space' || !store.activeSpace" class="flex-1" />
    <footer class="shrink-0 px-2.5 py-2">
      <button type="button" :class="rowClass" @click="store.recentlyDeletedSpaceId = null">
        <Trash2 class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.trash.recent.title") }}
      </button>
    </footer>
  </aside>
</template>
