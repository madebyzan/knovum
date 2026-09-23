import { computed, inject } from "vue";
import { useI18n } from "vue-i18n";

import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";
import { workspaceSidebarKey } from "../../../shared/contracts/workspace-sidebar";
import { useCanvasExchangeSession } from "../canvas-exchange/canvas-exchange-session";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { createStructureIndex, effectiveOperationRoots } from "../model/structure/structure-index";
import {
  canRenameStructureNode,
  dissolveGroupCapability,
  groupCapability,
} from "../organization/structure-capabilities";
import { useSpaceStore } from "../session/space-store";
import { useWorkspaceTabCommands } from "../workspace/tab-commands";
import { useSpaceLibraryCommands } from "../workspace/use-space-library-commands";
import { createCanvasEditingCommands } from "./canvas-editing-commands";
import { createCanvasExchangeCommands } from "./canvas-exchange-commands";
import { NUDGE_COMMANDS } from "./nudge-commands";
import { spacePreviewCommand } from "./space-preview-command";
import { useCanvasNavigationCommands } from "./use-canvas-navigation-commands";
import { videoEditCommands } from "./video-edit-commands";

export function useSpaceCommands() {
  const { t } = useI18n();
  const store = useSpaceStore();
  const sidebar = inject(workspaceSidebarKey)!;
  const { activeEditor, selection } = useCanvasEditorSession();
  const { activePort: activeExchange } = useCanvasExchangeSession();
  const workspaceTabCommands = useWorkspaceTabCommands();
  const libraryCommands = useSpaceLibraryCommands();
  const canvasNavigationCommands = useCanvasNavigationCommands();
  return computed<readonly AppCommand[]>(() => {
    const selectedIds = () => selection.value.nodeIds;
    const selectedStructureNodeIds = () => {
      const index = createStructureIndex(store.editSnapshot.structure);
      const contentStructureNodeIds = selectedIds().flatMap((nodeId) => {
        const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
        return structureNodeId ? [structureNodeId] : [];
      });
      return effectiveOperationRoots(
        [...selection.value.structureNodeIds, ...contentStructureNodeIds],
        index,
      ).map(({ id }) => id);
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
    if (store.videoEditorNodeId) return videoEditCommands(t);

    const hasStructureSelection = selectedStructureNodeIds().length > 0;
    const editingCommands: AppCommand[] = [
      {
        id: "spaces.group-selection",
        title: t("spaces.canvas.groupSelection"),
        shortcut: "$mod+G",
        category: "canvas",
        availability: () =>
          availableWhen(
            groupCapability(store.editSnapshot, selectedStructureNodeIds()).enabled,
            "selection_not_eligible",
          ),
        run: async () => {
          await store.executeSpaceEdit({
            kind: "group.create",
            groupId: crypto.randomUUID(),
            rootIds: selectedStructureNodeIds(),
            title: t("spaces.canvas.untitledGroup"),
          });
        },
      },
      {
        id: "spaces.dissolve-group",
        title: t("spaces.canvas.ungroupSelection"),
        shortcut: "$mod+Shift+G",
        category: "canvas",
        availability: () =>
          availableWhen(
            dissolveGroupCapability(store.editSnapshot, selectedStructureNodeIds()).enabled,
            "selection_not_eligible",
          ),
        run: async () => {
          await store.executeSpaceEdit({ kind: "group.dissolve", rootIds: selectedStructureNodeIds() });
        },
      },
      {
        id: "spaces.rename-node",
        title: t("spaces.navigation.renameNode"),
        shortcut: "F2",
        category: "canvas",
        availability: () =>
          availableWhen(
            canRenameStructureNode(store.editSnapshot, selectedStructureNodeIds()),
            "selection_not_eligible",
          ),
        run: () => {
          sidebar.reveal();
          store.outlineRenameStructureNodeId = selectedStructureNodeIds()[0]!;
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
        hasNodeSelection: () => selectedStructureNodeIds().length > 0,
        hasSelection: () => selectedStructureNodeIds().length > 0 || selection.value.relationIds.length > 0,
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
    const exchangeCommands = createCanvasExchangeCommands({
      exchange: () => activeExchange.value,
      spaceTitle: () => store.activeSpace?.title,
      importTitle: t("spaces.transfer.importJsonCanvas"),
      exportTitle: t("spaces.transfer.exportJsonCanvas"),
    });
    const nudgeCommands = hasStructureSelection
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
    const preview = spacePreviewCommand(store, selectedIds(), t("spaces.transfer.systemPreview"));
    return [
      ...workspaceTabCommands.value,
      ...editingCommands,
      ...exchangeCommands,
      ...nudgeCommands,
      ...canvasNavigationCommands.value,
      ...(preview ? [preview] : []),
    ];
  });
}
