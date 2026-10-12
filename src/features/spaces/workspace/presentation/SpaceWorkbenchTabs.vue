<script setup lang="ts">
import { Plus } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { isString } from "es-toolkit";
import { storeToRefs } from "pinia";
import { TabsList, TabsRoot } from "reka-ui";
import { nextTick, useTemplateRef, watch, watchEffect } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { toast } from "vue-sonner";

import { syncWorkspaceMenuState } from "../../../../foundation/desktop/workspace-menu";
import { formatErrorDiagnostic, getErrorMessage } from "../../../../foundation/errors/error-format";
import { isApplicationLeaveCancelled } from "../../../../shared/contracts/application-leave";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { useSpaceStore } from "../../session/space-store";
import SpaceLibraryToolbar from "./library/SpaceLibraryToolbar.vue";
import SpaceWorkbenchTab from "./SpaceWorkbenchTab.vue";
import { useWorkbenchTabMenu } from "./use-workbench-tab-menu";
import { useWorkbenchTabViews } from "./use-workbench-tab-views";

const store = useSpaceStore();
const route = useRoute();
const { activeTabId, closedTabs, permanentTabId, surface } = storeToRefs(store);
const { locale, t } = useI18n();
const tabViews = useWorkbenchTabViews();
const tabsViewport = useTemplateRef<HTMLDivElement>("tabsViewport");

function createSpace() {
  void store.createSpace(t("spaces.space.untitled")).catch(reportTabFailure);
}

/**
 * Opening a tab loads a work surface and commits the outgoing one, so the strip activates manually:
 * focus — from the secondary button that opens the tab menu or from roving arrow keys — only moves,
 * and Enter or Space opens the focused tab.
 */
function activateTab(tabId: unknown) {
  if (!isString(tabId)) throw new TypeError("Workbench tab identifiers must be strings");
  void store.activateTab(tabId).catch(reportTabFailure);
}

const showTabMenu = useWorkbenchTabMenu({
  createSpace,
  closeTab: (tabId) => void closeTabs(() => store.closeTab(tabId)),
  closeOtherTabs: (tabId) => void closeTabs(() => store.closeOtherTabs(tabId)),
  closeTabsToRight: (tabId) => void closeTabs(() => store.closeTabsToRight(tabId)),
});

/** Closing a tab keeps keyboard focus in the strip, on the tab that became active. */
async function closeTabs(operation: () => Promise<void>) {
  try {
    await operation();
    await nextTick();
    focusActiveTab();
  } catch (error) {
    reportTabFailure(error);
  }
}

function focusActiveTab() {
  tabsViewport.value?.querySelector<HTMLElement>('[role="tab"][data-state="active"]')?.focus();
}

function reportTabFailure(error: unknown) {
  if (isApplicationLeaveCancelled(error)) return;
  toast.error(`${t("spaces.space.tabs.actionFailed")}: ${getErrorMessage(error)}`);
  void logError(`Workbench tab action failed: ${formatErrorDiagnostic(error)}`);
}

watch(activeTabId, async (tabId) => {
  if (!tabId) return;
  await nextTick();
  tabsViewport.value
    ?.querySelector<HTMLElement>(`[data-tab-id="${CSS.escape(tabId)}"]`)
    ?.scrollIntoView({ block: "nearest", inline: "nearest" });
});

watchEffect(() => {
  void locale.value;
  void syncWorkspaceMenuState({
    workspaceActive: route.name === "spaces",
    canClose: Boolean(activeTabId.value) && activeTabId.value !== permanentTabId.value,
    canReopen: closedTabs.value.length > 0,
  }).catch((error) =>
    logError(`Failed to synchronize workspace menu state: ${formatErrorDiagnostic(error)}`),
  );
});
</script>

<template>
  <div class="flex min-w-0 flex-1 items-center gap-1">
    <SpaceLibraryToolbar v-if="surface.kind === 'library'" @create="createSpace" />
    <TabsRoot
      v-else
      activation-mode="manual"
      :model-value="activeTabId ?? ''"
      class="min-w-0 flex-1"
      @update:model-value="activateTab"
    >
      <div
        ref="tabsViewport"
        class="flex scrollbar-none items-center gap-1 overflow-x-auto py-1 [&::-webkit-scrollbar]:hidden"
      >
        <TabsList :aria-label="t('spaces.space.tabs.label')" class="flex w-max shrink-0 items-center gap-1">
          <SpaceWorkbenchTab
            v-for="view in tabViews"
            :key="view.id"
            :view="view"
            @close="closeTabs(() => store.closeTab(view.id))"
            @context-menu="showTabMenu(view)"
          />
        </TabsList>
        <span class="shrink-0">
          <UiIconButton :label="t('spaces.space.new')" shortcut="$mod+T" size="sm" @click="createSpace">
            <Plus class="size-3.5" :stroke-width="1.6" />
          </UiIconButton>
        </span>
      </div>
    </TabsRoot>
  </div>
</template>
