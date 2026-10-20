import { useI18n } from "vue-i18n";

import type { ContextMenuItem } from "../../../../foundation/context-menu/popup";
import { useSpaceStore } from "../../session/space-store";
import { popupSpaceContextMenu, spaceMenuItem } from "../space-menu";
import { useSpaceContextMenu } from "../use-space-context-menu";
import type { WorkbenchTabView } from "./use-workbench-tab-views";

interface WorkbenchTabMenuActions {
  createSpace: () => void;
  closeTab: (tabId: string) => void;
  closeOtherTabs: (tabId: string) => void;
  closeTabsToRight: (tabId: string) => void;
}

/** One context menu for every tab; Space tabs additionally offer the Space context menu groups. */
export function useWorkbenchTabMenu(actions: WorkbenchTabMenuActions) {
  const store = useSpaceStore();
  const { groups: spaceGroups } = useSpaceContextMenu();
  const { t } = useI18n();

  function tabActions(view: WorkbenchTabView): readonly ContextMenuItem[] {
    const closableTabs = store.tabs.filter(({ id }) => id !== store.permanentTabId);
    const index = store.tabs.findIndex(({ id }) => id === view.id);
    return [
      spaceMenuItem(t("spaces.space.new"), actions.createSpace),
      spaceMenuItem(t("spaces.space.tabs.close"), () => actions.closeTab(view.id), {
        enabled: view.canClose,
      }),
      spaceMenuItem(t("spaces.space.tabs.closeOthers"), () => actions.closeOtherTabs(view.id), {
        enabled: closableTabs.some(({ id }) => id !== view.id),
      }),
      spaceMenuItem(t("spaces.space.tabs.closeToRight"), () => actions.closeTabsToRight(view.id), {
        enabled: store.tabs.slice(index + 1).some((tab) => tab.id !== store.permanentTabId),
      }),
    ];
  }

  return (view: WorkbenchTabView) => {
    const space = store.spaces.find((candidate) => candidate.id === view.spaceId);
    const groups = [tabActions(view)];
    if (view.kind === "space" && space) groups.push(...spaceGroups(space));
    return popupSpaceContextMenu(groups);
  };
}
