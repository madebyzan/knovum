import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { useSpaceStore } from "../session/space-store";

/** Every tab shortcut acts on the single workbench tab list, whatever a tab contains. */
export function useWorkbenchTabCommands() {
  const store = useSpaceStore();
  const { t } = useI18n();

  function cycleCommands(): AppCommand[] {
    if (store.tabs.length < 2) return [];
    return ([-1, 1] as const).map((direction) => ({
      id: direction === 1 ? "spaces.next-tab" : "spaces.previous-tab",
      title: t(direction === 1 ? "spaces.space.tabs.next" : "spaces.space.tabs.previous"),
      shortcut: direction === 1 ? "Control+Tab" : "Control+Shift+Tab",
      allowInEditable: true,
      category: "navigation",
      run: () => {
        const index = store.tabs.findIndex(({ id }) => id === store.activeTabId);
        const next = store.tabs[(index + direction + store.tabs.length) % store.tabs.length];
        return next ? store.activateTab(next.id) : Promise.resolve();
      },
    }));
  }

  function numberedCommands(): AppCommand[] {
    const numbered = store.tabs.slice(0, 8);
    if (store.tabs.length > 8) numbered.push(store.tabs.at(-1)!);
    return numbered.map(({ id }, index) => ({
      id: `spaces.switch-tab-${index + 1}`,
      title: t("spaces.space.tabs.switchTo", { number: index + 1 }),
      shortcut: `$mod+${index + 1}`,
      allowInEditable: true,
      category: "navigation",
      run: () => store.activateTab(id),
    }));
  }

  return computed<readonly AppCommand[]>(() => {
    const commands: AppCommand[] = [
      {
        id: "spaces.new-space",
        title: t("spaces.space.new"),
        shortcut: "$mod+T",
        appMenuShortcut: true,
        allowInEditable: true,
        category: "navigation",
        run: () => store.createSpace(t("spaces.space.untitled")),
      },
    ];
    if (store.closedTabs.length) {
      commands.push({
        id: "spaces.reopen-closed-tab",
        title: t("spaces.space.tabs.reopen"),
        shortcut: "$mod+Shift+T",
        appMenuShortcut: true,
        allowInEditable: true,
        category: "navigation",
        run: store.reopenClosedTab,
      });
    }
    const activeTabId = store.activeTabId;
    if (activeTabId && activeTabId !== store.permanentTabId) {
      commands.push({
        id: "spaces.close-tab",
        title: t("spaces.space.tabs.close"),
        shortcut: "$mod+W",
        appMenuShortcut: true,
        allowInEditable: true,
        category: "navigation",
        run: () => store.closeTab(activeTabId),
      });
    }
    commands.push(...cycleCommands(), ...numberedCommands());
    if (store.canNavigateBack) {
      commands.push({
        id: "spaces.navigate-back",
        title: t("spaces.space.back"),
        shortcut: "Alt+ArrowLeft",
        category: "navigation",
        run: () => store.navigateWorkbenchHistory(-1),
      });
    }
    if (store.canNavigateForward) {
      commands.push({
        id: "spaces.navigate-forward",
        title: t("spaces.space.forward"),
        shortcut: "Alt+ArrowRight",
        category: "navigation",
        run: () => store.navigateWorkbenchHistory(1),
      });
    }
    return commands;
  });
}
