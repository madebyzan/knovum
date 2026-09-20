import { computed, inject } from "vue";
import { useI18n } from "vue-i18n";

import { openSystemPreview } from "../../../foundation/system-preview/system-preview";
import type { AppCommand } from "../../../shared/contracts/app-command";
import { workspaceSidebarKey } from "../../../shared/contracts/workspace-sidebar";
import { useCanvasExchangeSession } from "../canvas-exchange/canvas-exchange-session";
import { exportJsonCanvasFile, importJsonCanvasFile } from "../canvas-exchange/transfer";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import { useSpaceStore } from "../session/space-store";
import { useWorkspaceTabCommands } from "../workspace/tab-commands";
import { useSpaceLibraryCommands } from "../workspace/use-space-library-commands";
import { createCanvasEditingCommands } from "./canvas-editing-commands";
import { NUDGE_COMMANDS } from "./nudge-commands";
import { useStructureCommands } from "./structure-commands";
import { useCanvasNavigationCommands } from "./use-canvas-navigation-commands";

export function useSpaceCommands() {
  const { t } = useI18n();
  const store = useSpaceStore();
  const sidebar = inject(workspaceSidebarKey)!;
  const { activeEditor, selection } = useCanvasEditorSession();
  const { activePort: activeExchange } = useCanvasExchangeSession();
  const workspaceTabCommands = useWorkspaceTabCommands();
  const libraryCommands = useSpaceLibraryCommands();
  const canvasNavigationCommands = useCanvasNavigationCommands();
  const structureCommands = useStructureCommands();
  return computed<readonly AppCommand[]>(() => {
    const selectedIds = () => selection.value.nodeIds;
    const previewPath = () => {
      const selected = new Set(selectedIds());
      const node = store.nodes.find(
        (candidate) => selected.has(candidate.id) && isSpaceManagedAssetPayload(candidate.data),
      );
      return node && isSpaceManagedAssetPayload(node.data) ? node.data.assetPath : undefined;
    };
    const searchAllCommand: AppCommand = {
      id: "spaces.find-all",
      title: t("spaces.navigation.searchAllSpaces"),
      shortcut: "$mod+Shift+F",
      category: "canvas",
      run: () => {
        store.globalFindOpen = true;
      },
    };
    if (store.workspaceMode === "library") return [...workspaceTabCommands.value, ...libraryCommands.value];
    if (!store.activeSpaceId) return [...workspaceTabCommands.value, searchAllCommand];

    const hasNodeSelection = selection.value.nodeIds.length > 0;
    const editingCommands: AppCommand[] = [
      {
        id: "spaces.rename-node",
        title: t("spaces.navigation.renameNode"),
        shortcut: "F2",
        category: "canvas",
        isEnabled: () =>
          selectedIds().length === 1 && store.nodes.some((node) => node.id === selectedIds()[0]),
        run: () => {
          sidebar.reveal();
          store.outlineRenameNodeId = selectedIds()[0]!;
        },
      },
      {
        id: "spaces.focus-outline",
        title: t("spaces.navigation.outline"),
        shortcut: "$mod+Shift+O",
        category: "canvas",
        run: () => {
          store.outlineFocusRequest += 1;
        },
      },
      {
        id: "spaces.find",
        title: t("spaces.navigation.findInSpace"),
        shortcut: "$mod+F",
        category: "canvas",
        run: () => {
          store.findOpen = true;
        },
      },
      searchAllCommand,
      ...createCanvasEditingCommands({
        canRedo: () => store.canRedo,
        canUndo: () => store.canUndo,
        editor: () => activeEditor.value,
        hasNodeSelection: () => selection.value.nodeIds.length > 0,
        hasSelection: () => selection.value.nodeIds.length > 0 || selection.value.relationIds.length > 0,
        labels: {
          copy: t("common.copy"),
          cut: t("common.cut"),
          deleteSelection: t("spaces.navigation.deleteSelection"),
          duplicate: t("spaces.navigation.duplicateSelection"),
          paste: t("common.paste"),
          redo: t("common.redo"),
          selectAll: t("spaces.canvas.actions.selectAll"),
          undo: t("common.undo"),
        },
        redo: () => store.redo(),
        undo: () => store.undo(),
      }),
    ];
    const exchangeCommands: AppCommand[] = [
      {
        id: "spaces.import-json-canvas",
        title: t("spaces.transfer.importJsonCanvas"),
        category: "canvas",
        run: async () => {
          if (!activeExchange.value) throw new Error("Canvas exchange is not available");
          await importJsonCanvasFile(activeExchange.value);
        },
      },
      {
        id: "spaces.export-json-canvas",
        title: t("spaces.transfer.exportJsonCanvas"),
        category: "canvas",
        run: async () => {
          if (!activeExchange.value || !store.activeSpace)
            throw new Error("Canvas exchange is not available");
          await exportJsonCanvasFile(activeExchange.value, store.activeSpace.title);
        },
      },
    ];
    const nudgeCommands = hasNodeSelection
      ? NUDGE_COMMANDS.map<AppCommand>((input) => ({
          id: input.id,
          title: t(input.titleKey),
          shortcut: input.shortcut,
          shortcutListing: {
            groupId: input.shortcut.includes("Shift") ? "canvas.nudge-large" : "canvas.nudge",
            title: t(input.shortcut.includes("Shift") ? "shell.nudgeLarge" : "shell.nudgeSelection"),
            shortcut: input.shortcut.includes("Shift") ? "Shift+Arrow keys" : "Arrow keys",
          },
          category: "canvas",
          run: () => activeEditor.value?.nudgeSelection({ x: input.x, y: input.y }),
        }))
      : [];
    const previewCommands: AppCommand[] = previewPath()
      ? [
          {
            id: "spaces.system-preview",
            title: t("spaces.transfer.systemPreview"),
            category: "canvas",
            run: () => openSystemPreview(previewPath()!),
          },
        ]
      : [];
    return [
      ...workspaceTabCommands.value,
      ...editingCommands,
      ...exchangeCommands,
      ...nudgeCommands,
      ...canvasNavigationCommands.value,
      ...structureCommands.value,
      ...previewCommands,
    ];
  });
}
