<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { storeToRefs } from "pinia";
import { TabsList, TabsRoot } from "reka-ui";
import { computed, nextTick, useTemplateRef, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";

import { syncWorkspaceMenuState } from "../../../foundation/desktop/workspace-menu";
import UiIconButton from "../../../shared/ui/UiIconButton.vue";
import type { Space } from "../model/space";
import { useSpaceStore } from "../session/space-store";
import { spaceMenuItem } from "../workspace/space-menu";
import { useSpaceManagementMenu } from "../workspace/use-space-management-menu";
import SpaceLibraryToolbar from "./library/SpaceLibraryToolbar.vue";
import SpaceWorkspaceTab from "./SpaceWorkspaceTab.vue";

const store = useSpaceStore();
const management = useSpaceManagementMenu();
const route = useRoute();
const { spaces, draftSpace, activeSpaceId, openSpaceIds, closedSpaceIds, workspaceMode } = storeToRefs(store);
const { locale, t } = useI18n();
const tabsViewport = useTemplateRef<HTMLDivElement>("tabsViewport");
const openSpaces = computed(() =>
  openSpaceIds.value.flatMap((id) => {
    const space =
      spaces.value.find((candidate) => candidate.id === id && candidate.archivedAt === null) ??
      (draftSpace.value?.id === id ? draftSpace.value : undefined);
    return space ? [space] : [];
  }),
);

watch(activeSpaceId, async (id) => {
  if (!id) return;
  await nextTick();
  tabsViewport.value
    ?.querySelector<HTMLElement>(`[data-space-id="${CSS.escape(id)}"]`)
    ?.scrollIntoView({ block: "nearest", inline: "nearest" });
});

watchEffect(() => {
  void locale.value;
  void syncWorkspaceMenuState({
    workspaceActive: route.name === "spaces",
    canClose: openSpaceIds.value.length > 1,
    canReopen: closedSpaceIds.value.length > 0,
  }).catch((error) => logError(`Failed to synchronize workspace menu state: ${String(error)}`));
});

function createSpace() {
  void store.createSpace(t("spaces.space.untitledSpace")).catch(reportTabFailure);
}

function activateTab(id: unknown) {
  if (typeof id !== "string") throw new TypeError("Workspace tab identifiers must be strings");
  void store.openSpace(id).catch(reportTabFailure);
}

function closeTab(id: string) {
  void store.closeSpaceTab(id).catch(reportTabFailure);
}

function closeOtherTabs(id: string) {
  void store.closeOtherSpaceTabs(id).catch(reportTabFailure);
}

function closeTabsToRight(id: string) {
  void store.closeSpaceTabsToRight(id).catch(reportTabFailure);
}

function showTabMenu(space: Space) {
  void management.show(space, [
    [
      spaceMenuItem(t("spaces.space.newSpace"), createSpace),
      spaceMenuItem(t("spaces.space.closeTab"), () => closeTab(space.id), {
        enabled: openSpaceIds.value.length > 1,
      }),
      spaceMenuItem(t("spaces.space.closeOtherTabs"), () => closeOtherTabs(space.id), {
        enabled: openSpaceIds.value.length > 1,
      }),
      spaceMenuItem(t("spaces.space.closeTabsToRight"), () => closeTabsToRight(space.id), {
        enabled: openSpaceIds.value.indexOf(space.id) < openSpaceIds.value.length - 1,
      }),
    ],
  ]);
}

// Template handlers drop the promise a tab action returns, and a rejected action leaves the tab bar
// visibly unchanged, so the reason belongs in the application log rather than in an unhandled warning.
function reportTabFailure(error: unknown) {
  void logError(`Workspace tab action failed: ${String(error)}`);
}
</script>

<template>
  <div class="flex min-w-0 flex-1 items-center gap-1">
    <SpaceLibraryToolbar v-if="workspaceMode === 'library'" @create="createSpace" />
    <TabsRoot
      v-else
      :model-value="activeSpaceId ?? ''"
      class="min-w-0 flex-1"
      @update:model-value="activateTab"
    >
      <div
        ref="tabsViewport"
        class="flex scrollbar-none items-center gap-1 overflow-x-auto py-1 [&::-webkit-scrollbar]:hidden"
      >
        <TabsList :aria-label="t('spaces.space.openSpaces')" class="flex w-max shrink-0 items-center gap-1">
          <SpaceWorkspaceTab
            v-for="space in openSpaces"
            :key="space.id"
            :id="space.id"
            :title="space.title"
            :active="activeSpaceId === space.id"
            :can-close="openSpaceIds.length > 1"
            @close="closeTab(space.id)"
            @context-menu="showTabMenu(space)"
          />
        </TabsList>
        <UiIconButton
          class="shrink-0"
          :label="t('spaces.space.newSpace')"
          shortcut="$mod+T"
          size="sm"
          @click="createSpace"
        >
          <Plus class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
        </UiIconButton>
      </div>
    </TabsRoot>
  </div>
</template>
