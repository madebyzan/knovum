import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AppCommand } from "../../../shared/contracts/app-command";
import { useSpaceStore } from "../session/space-store";

export function useWorkspaceTabCommands() {
  const store = useSpaceStore();
  const { t } = useI18n();

  const selectedEditorId = computed(() => {
    if (!store.activeSpaceId || store.surface.kind === "library") return undefined;
    return store.activeWorkItems.get(store.activeSpaceId);
  });

  function workTabCommands(
    activeSpaceId: string | undefined,
    activeEditorId: string | undefined,
  ): AppCommand[] {
    if (!activeSpaceId || store.surface.kind === "library") return [];
    const items = [
      undefined,
      ...store.editorTabs.filter((tab) => tab.spaceId === activeSpaceId).map(({ id }) => id),
    ];
    if (items.length < 2) return [];
    return ([-1, 1] as const).map((direction) => ({
      id: direction === 1 ? "spaces.next-work-tab" : "spaces.previous-work-tab",
      title: t(direction === 1 ? "spaces.editor.tabs.next" : "spaces.editor.tabs.previous"),
      shortcut: direction === 1 ? "Control+Tab" : "Control+Shift+Tab",
      allowInEditable: true,
      category: "navigation",
      run: () => {
        const index = items.indexOf(activeEditorId);
        const next = items[(index + direction + items.length) % items.length];
        return next ? store.activateEditorTab(next) : store.returnToCanvas();
      },
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
    const activeEditorId = selectedEditorId.value;
    if (activeSpaceId && (activeEditorId || store.openSpaceIds.length > 1)) {
      commands.push({
        id: "spaces.close-tab",
        title: t(activeEditorId ? "common.close" : "spaces.space.tabs.close"),
        shortcut: "$mod+W",
        appMenuShortcut: true,
        allowInEditable: true,
        category: "navigation",
        run: () =>
          activeEditorId ? store.closeEditorTab(activeEditorId) : store.closeSpaceTab(activeSpaceId),
      });
    }
    commands.push(...workTabCommands(activeSpaceId, activeEditorId));
    const numberedTabs = store.openSpaceIds.slice(0, 8);
    if (store.openSpaceIds.length > 8) numberedTabs.push(store.openSpaceIds.at(-1)!);
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
