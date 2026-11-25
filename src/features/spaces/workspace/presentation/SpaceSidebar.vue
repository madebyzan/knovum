<script setup lang="ts">
import { Clock3, Inbox, LayoutGrid, Settings, Star } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { RouterLink } from "vue-router";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { getUpdateAttentionVersion } from "../../../../foundation/updates/update-presentation";
import { useUpdateStore } from "../../../../foundation/updates/update-store";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import SpaceSidebarOutline from "../../outline/presentation/SpaceSidebarOutline.vue";
import { useSpaceStore } from "../../session/space-store";
import { useSpaceContextMenu } from "../use-space-context-menu";

const store = useSpaceStore();
const updates = useUpdateStore();
const { show: showSpaceMenu } = useSpaceContextMenu();
const { spaces, activeSpaceId, surface, libraryScope } = storeToRefs(store);
const { t } = useI18n();
const updateBadgeVersion = computed(() => getUpdateAttentionVersion(updates.state));
const ROW_CLASS =
  "flex h-interaction w-full items-center gap-2 rounded-md px-2 text-left text-label text-fg-secondary hover:bg-(--knovum-sidebar-item-bg-hover) hover:text-fg-primary data-[active=true]:bg-(--knovum-sidebar-item-bg-selected) data-[active=true]:text-fg-primary";
const inbox = computed(() => spaces.value.find((space) => space.kind === "inbox"));
function navigate(operation: () => Promise<void>) {
  void operation().catch((error: unknown) => {
    if (!isApplicationLeaveCancelled(error)) toast.error(getErrorMessage(error));
  });
}
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
        :data-active="surface.kind !== 'library' && activeSpaceId === inbox.id"
        :class="ROW_CLASS"
        @click="navigate(() => store.openSpace(inbox!.id))"
        @contextmenu.prevent="showSpaceMenu(inbox)"
      >
        <Inbox class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.inbox") }}
      </button>
      <button
        type="button"
        :class="ROW_CLASS"
        :data-active="surface.kind === 'library' && libraryScope === 'all'"
        @click="navigate(() => store.showSpaceLibrary('all'))"
      >
        <LayoutGrid class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.sections.all") }}
      </button>
      <button
        type="button"
        :class="ROW_CLASS"
        :data-active="surface.kind === 'library' && libraryScope === 'recent'"
        @click="navigate(() => store.showSpaceLibrary('recent'))"
      >
        <Clock3 class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.sections.recent") }}
      </button>
      <button
        type="button"
        :class="ROW_CLASS"
        :data-active="surface.kind === 'library' && libraryScope === 'favorites'"
        @click="navigate(() => store.showSpaceLibrary('favorites'))"
      >
        <Star class="size-3.5" :stroke-width="1.55" />
        {{ t("spaces.space.sections.favorites") }}
      </button>
    </nav>
    <SpaceSidebarOutline v-if="store.workbench.outline" />
    <div v-if="!store.workbench.outline" class="flex-1" />
    <footer class="shrink-0 px-2.5 py-2">
      <RouterLink :to="{ name: 'settings' }" :class="ROW_CLASS">
        <Settings class="size-3.5" :stroke-width="1.55" />
        <span class="min-w-0 flex-1 truncate">{{ t("common.settings") }}</span>
        <span
          v-if="updateBadgeVersion"
          data-testid="sidebar-settings-update-badge"
          class="shrink-0 text-caption-sm font-medium text-fg-accent tabular-nums"
        >
          {{ updateBadgeVersion }}
        </span>
      </RouterLink>
    </footer>
  </aside>
</template>
