import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AppCommand } from "../../../shared/contracts/app-command";
import type { WorkspaceCommandContext } from "../../../shared/contracts/workspace-definition";
import { activeCanvasExchangePort as activeExchange } from "../canvas-exchange/canvas-exchange-session";
import { getCanvasEditorSession } from "../canvas/editor-session";
import { currentSelectionRoots } from "../canvas/selection/selection-roots";
import { useNodeOperationGate } from "../canvas/use-node-operation-gate";
import { createStructureIndex } from "../model/structure/structure-index";
import { useSpaceStore } from "../session/space-store";
import { useSpaceLibraryCommands } from "../workspace/use-space-library-commands";
import { useWorkspaceTabCommands } from "../workspace/use-workspace-tab-commands";
import { createCanvasEditingCommands } from "./canvas-editing-commands";
import { createCanvasExchangeCommands } from "./canvas-exchange-commands";
import {
  guardAllCommands,
  guardCanvasExchangeCommands,
  guardNavigationCommands,
  guardSelectionCommands,
} from "./node-operation-command-policy";
import { NUDGE_COMMANDS } from "./nudge-commands";
import { spacePreviewCommand } from "./space-preview-command";
import { createSpaceStructureCommands } from "./space-structure-commands";
import { useCanvasNavigationCommands } from "./use-canvas-navigation-commands";
import { videoEditCommands } from "./video-edit-commands";

export function useSpaceCommands({ sidebar, rightSidebar }: WorkspaceCommandContext) {
  const { t } = useI18n();
  const store = useSpaceStore();
  const { activeEditor, selection } = getCanvasEditorSession();
  const nodeOperations = useNodeOperationGate();
  const workspaceTabCommands = useWorkspaceTabCommands();
  const libraryCommands = useSpaceLibraryCommands();
  const canvasNavigationCommands = useCanvasNavigationCommands(rightSidebar);
  const selectedIds = () => selection.value.nodeIds;
  return computed<readonly AppCommand[]>(() => {
    const selectedStructureNodeIds = () => {
      const index = createStructureIndex(store.editSnapshot.structure);
      return currentSelectionRoots(selectedIds(), selection.value.structureNodeIds, index).map(
        ({ id }) => id,
      );
    };
    const searchAllCommand: AppCommand = {
      id: "spaces.find-all",
      title: t("spaces.search.allSpaces.title"),
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
      ...createSpaceStructureCommands({ store, selectedStructureNodeIds, t, sidebar }),
      {
        id: "spaces.focus-outline",
        title: t("spaces.sidebar.outline.title"),
        shortcut: "$mod+Shift+O",
        category: "canvas",
        run: () => {
          store.outlineFocusRequest += 1;
        },
      },
      {
        id: "spaces.find",
        title: t("spaces.search.find.title"),
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
          deleteSelection: t("spaces.canvas.selection.delete"),
          duplicate: t("common.duplicate"),
          paste: t("common.paste"),
          redo: t("common.redo"),
          selectAll: t("spaces.canvas.selection.selectAll"),
          undo: t("common.undo"),
        },
        redo: () => store.redo(),
        undo: () => store.undo(),
      }),
    ];
    const exchangeCommands = createCanvasExchangeCommands({
      exchange: () => activeExchange.value,
      spaceTitle: () => store.activeSpace?.title,
      importTitle: t("spaces.importExport.jsonCanvas.import"),
      exportTitle: t("spaces.importExport.jsonCanvas.export"),
    });
    const nudgeCommands = hasStructureSelection
      ? NUDGE_COMMANDS.map<AppCommand>((input) => ({
          id: input.id,
          title: t(input.titleKey),
          shortcut: input.shortcut,
          shortcutListing: {
            groupId: input.shortcut.includes("Shift") ? "canvas.nudge-large" : "canvas.nudge",
            title: t(
              input.shortcut.includes("Shift")
                ? "spaces.canvas.shortcuts.nudgeLarge"
                : "spaces.canvas.shortcuts.nudgeSelection",
            ),
            shortcut: input.shortcut.includes("Shift") ? "Shift+Arrow keys" : "Arrow keys",
          },
          category: "canvas",
          run: () => activeEditor.value?.nudgeSelection({ x: input.x, y: input.y }),
        }))
      : [];
    const preview = spacePreviewCommand(store, selectedIds(), t("spaces.importExport.preview.system"));
    return [
      ...workspaceTabCommands.value,
      ...guardSelectionCommands(editingCommands, nodeOperations.allows),
      ...guardCanvasExchangeCommands(exchangeCommands, nodeOperations.allows),
      ...guardAllCommands(nudgeCommands, "structure", nodeOperations.allows),
      ...guardNavigationCommands(canvasNavigationCommands.value, nodeOperations.allows),
      ...(preview ? [preview] : []),
    ];
  });
}
