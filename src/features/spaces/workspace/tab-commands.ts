import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { useSpaceStore } from "../session/space-store";

export function useWorkspaceTabCommands() {
  const store = useSpaceStore();
  const { t } = useI18n();

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
    if (store.closedSpaceIds.length) {
      commands.push({
        id: "spaces.reopen-closed-tab",
        title: t("spaces.space.tabs.reopen"),
        shortcut: "$mod+Shift+T",
        appMenuShortcut: true,
        allowInEditable: true,
        category: "navigation",
        run: store.reopenClosedSpaceTab,
      });
    }
    const activeSpaceId = store.activeSpaceId;
    if (activeSpaceId && store.openSpaceIds.length > 1) {
      commands.push({
        id: "spaces.close-tab",
        title: t("spaces.space.tabs.close"),
        shortcut: "$mod+W",
        appMenuShortcut: true,
        allowInEditable: true,
        category: "navigation",
        run: () => store.closeSpaceTab(activeSpaceId),
      });
    }
    const numberedTabs = [
      ...store.openSpaceIds.slice(0, 8),
      ...(store.openSpaceIds.length > 8 ? [store.openSpaceIds.at(-1)] : []),
    ].filter((id): id is string => Boolean(id));
    numberedTabs.forEach((id, index) => {
      const number = index + 1;
      commands.push({
        id: `spaces.switch-tab-${number}`,
        title: t("spaces.space.tabs.switchTo", { number }),
        shortcut: `$mod+${number}`,
        allowInEditable: true,
        category: "navigation",
        run: () => store.openSpace(id),
      });
    });
    if (store.canNavigateBack) {
      commands.push({
        id: "spaces.navigate-back",
        title: t("spaces.space.back"),
        shortcut: "Alt+ArrowLeft",
        category: "navigation",
        run: () => store.navigateSpaceHistory(-1),
      });
    }
    if (store.canNavigateForward) {
      commands.push({
        id: "spaces.navigate-forward",
        title: t("spaces.space.forward"),
        shortcut: "Alt+ArrowRight",
        category: "navigation",
        run: () => store.navigateSpaceHistory(1),
      });
    }
    return commands;
  });
}
