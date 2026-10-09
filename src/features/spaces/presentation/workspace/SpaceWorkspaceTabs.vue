<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { isString } from "es-toolkit";
import { storeToRefs } from "pinia";
import { TabsList, TabsRoot } from "reka-ui";
import { computed, nextTick, useTemplateRef, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { toast } from "vue-sonner";

import { syncWorkspaceMenuState } from "../../../../foundation/desktop/workspace-menu";
import { formatErrorDiagnostic, getErrorMessage } from "../../../../foundation/errors/error-format";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import type { Space } from "../../model/space";
import { useSpaceStore } from "../../session/space-store";
import { spaceMenuItem } from "../../workspace/space-menu";
import { useSpaceManagementMenu } from "../../workspace/use-space-management-menu";
import SpaceLibraryToolbar from "../library/SpaceLibraryToolbar.vue";
import SpaceWorkspaceTab from "./SpaceWorkspaceTab.vue";

const store = useSpaceStore();
const management = useSpaceManagementMenu();
const route = useRoute();
const { spaces, draftSpace, activeSpaceId, openSpaceIds, closedSpaceIds, workspaceMode } = storeToRefs(store);
const { locale, t } = useI18n();
const tabsViewport = useTemplateRef<HTMLDivElement>("tabsViewport");
const newSpaceButtonHost = useTemplateRef<HTMLElement>("newSpaceButtonHost");
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
  }).catch((error) =>
    logError(`Failed to synchronize workspace menu state: ${formatErrorDiagnostic(error)}`),
  );
});

function createSpace() {
  void store.createSpace(t("spaces.space.untitled")).catch(reportTabFailure);
}

function activateTab(id: unknown) {
  if (!isString(id)) throw new TypeError("Workspace tab identifiers must be strings");
  void store.openSpace(id).catch(reportTabFailure);
}

function closeTab(id: string) {
  void closeTabs(() => store.closeSpaceTab(id));
}

function closeOtherTabs(id: string) {
  void closeTabs(() => store.closeOtherSpaceTabs(id));
}

function closeTabsToRight(id: string) {
  void closeTabs(() => store.closeSpaceTabsToRight(id));
}

async function closeTabs(operation: () => Promise<void>) {
  try {
    await operation();
    await nextTick();
    const activeId = store.activeSpaceId;
    if (activeId) {
      tabsViewport.value
        ?.querySelector<HTMLElement>(`[data-space-id="${CSS.escape(activeId)}"] [role="tab"]`)
        ?.focus();
    } else {
      newSpaceButtonHost.value?.querySelector<HTMLElement>("button")?.focus();
    }
  } catch (error) {
    reportTabFailure(error);
  }
}

function showTabMenu(space: Space) {
  void management.show(space, [
    [
      spaceMenuItem(t("spaces.space.new"), createSpace),
      spaceMenuItem(t("spaces.space.tabs.close"), () => closeTab(space.id), {
        enabled: openSpaceIds.value.length > 1,
      }),
      spaceMenuItem(t("spaces.space.tabs.closeOthers"), () => closeOtherTabs(space.id), {
        enabled: openSpaceIds.value.length > 1,
      }),
      spaceMenuItem(t("spaces.space.tabs.closeToRight"), () => closeTabsToRight(space.id), {
        enabled: openSpaceIds.value.indexOf(space.id) < openSpaceIds.value.length - 1,
      }),
    ],
  ]);
}

function reportTabFailure(error: unknown) {
  if (isApplicationLeaveCancelled(error)) return;
  toast.error(`${t("spaces.space.tabs.actionFailed")}: ${getErrorMessage(error)}`);
  void logError(`Workspace tab action failed: ${formatErrorDiagnostic(error)}`);
}
</script>

<template>
  <div class="flex min-w-0 flex-1 items-center gap-1">
    <div v-if="workspaceMode === 'library'" ref="newSpaceButtonHost" class="contents">
      <SpaceLibraryToolbar @create="createSpace" />
    </div>
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
        <TabsList
          :aria-label="t('spaces.space.sections.open')"
          class="flex w-max shrink-0 items-center gap-1"
        >
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
        <span ref="newSpaceButtonHost" class="shrink-0">
          <UiIconButton :label="t('spaces.space.new')" shortcut="$mod+T" size="sm" @click="createSpace">
            <Plus class="size-3.5" :stroke-width="1.6" />
          </UiIconButton>
        </span>
      </div>
    </TabsRoot>
  </div>
</template>
