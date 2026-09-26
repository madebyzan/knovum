import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand, CommandCategory } from "../../../shared/contracts/app-command";
import { useCanvasToolStore } from "../canvas/canvas-tool-store";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { currentSelectionRoots } from "../canvas/selection/selection-roots";
import { useContextDock } from "../canvas/use-context-dock";
import { isSpaceSemanticNode } from "../knowledge/model/semantic-node";
import { ancestorsOf, createStructureIndex } from "../model/structure/structure-index";
import { useSceneList } from "../scenes/use-scene-list";
import { useSpaceStore } from "../session/space-store";
import { CANVAS_TOOL_COMMANDS } from "./canvas-tool-commands";
import { SPACE_COMMAND_KEYBINDINGS, SPACE_COMMAND_SHORTCUTS } from "./space-command-shortcuts";

export function useCanvasNavigationCommands() {
  const { t } = useI18n();
  const store = useSpaceStore();
  const dock = useContextDock();
  const session = useCanvasEditorSession();
  const tools = useCanvasToolStore();
  const scenes = useSceneList();
  const selectedNodes = () => {
    const selected = new Set(session.selection.value.nodeIds);
    return store.nodes.filter(({ id }) => selected.has(id));
  };
  const selectedStructureNodeIds = () => {
    const index = createStructureIndex(store.editSnapshot.structure);
    return currentSelectionRoots(
      session.selection.value.nodeIds,
      session.selection.value.structureNodeIds,
      index,
    ).map(({ id }) => id);
  };

  return computed<readonly AppCommand[]>(() => [
    ...CANVAS_TOOL_COMMANDS.map((toolCommand) =>
      command(`spaces.tool-${toolCommand.tool}`, t(toolCommand.title), {
        shortcut: toolCommand.shortcut,
        run: () => tools.selectTool(toolCommand.tool),
      }),
    ),
    command("spaces.cancel-canvas-interaction", t("spaces.canvas.cancelInteraction"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.cancelInteraction,
      run: () =>
        tools.tool !== "select" ? tools.resetTool() : session.activeEditor.value?.cancelInteraction(),
    }),
    command("spaces.zoom-in", t("spaces.canvas.zoom.in"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.zoomIn,
      keybindings: SPACE_COMMAND_KEYBINDINGS.zoomIn,
      run: () => session.activeEditor.value?.zoomIn(),
    }),
    command("spaces.zoom-out", t("spaces.canvas.zoom.out"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.zoomOut,
      run: () => session.activeEditor.value?.zoomOut(),
    }),
    command("spaces.zoom-reset", t("spaces.canvas.shortcuts.zoomReset"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.resetZoom,
      run: () => session.activeEditor.value?.setZoom(1),
    }),
    command("spaces.zoom-to-fit", t("spaces.canvas.zoom.toFitContent"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.zoomToFit,
      keybindings: SPACE_COMMAND_KEYBINDINGS.zoomToFit,
      run: () => session.activeEditor.value?.zoomToFit(),
    }),
    command("spaces.zoom-to-selection", t("spaces.canvas.zoom.toSelection"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.zoomToSelection,
      keybindings: SPACE_COMMAND_KEYBINDINGS.zoomToSelection,
      availability: () => {
        const selection = session.selection.value;
        return availableWhen(
          Boolean(
            selection.nodeIds.length || selection.structureNodeIds.length || selection.relationIds.length,
          ),
          "no_selection",
        );
      },
      run: () => session.activeEditor.value?.zoomToSelection(),
    }),
    command("spaces.previous-scene", t("spaces.scenes.previous"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.previousScene,
      category: "navigation",
      availability: () => availableWhen(scenes.scenes.value.length > 1, "not_enough_scenes"),
      run: () => scenes.stepScene(-1),
    }),
    command("spaces.next-scene", t("spaces.scenes.next"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.nextScene,
      category: "navigation",
      availability: () => availableWhen(scenes.scenes.value.length > 1, "not_enough_scenes"),
      run: () => scenes.stepScene(1),
    }),
    command("spaces.ask", t("spaces.assistant.evidence.ask"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.ask,
      run: () => dock.openAsk("space", { restart: true }),
    }),
    command("spaces.focus-selection", t("spaces.scenes.focus.selection"), {
      availability: () => availableWhen(selectedNodes().length > 0, "no_selection"),
      run: () => {
        const nodes = selectedNodes();
        if (nodes[0]) store.focusNeighborhood(nodes[0].id);
      },
    }),
    command("spaces.connect-selection", t("spaces.canvas.relations.connect"), {
      availability: () =>
        availableWhen(
          selectedNodes().length === 2 && selectedNodes().filter(isSpaceSemanticNode).length === 2,
          "selection_not_eligible",
        ),
      run: async () => {
        const [source, target] = selectedNodes().filter(isSpaceSemanticNode);
        if (source && target) await store.createRelation(source.id, target.id, "related");
      },
    }),
    command("spaces.toggle-selection-lock", t("spaces.canvas.selection.toggleLock"), {
      shortcut: SPACE_COMMAND_SHORTCUTS.lockSelection,
      category: "editing",
      availability: () => {
        const index = createStructureIndex(store.editSnapshot.structure);
        const structureNodeIds = selectedStructureNodeIds();
        return availableWhen(
          structureNodeIds.length > 0 &&
            structureNodeIds.every((id) => !ancestorsOf(id, index).some(({ locked }) => locked)),
          "selection_not_eligible",
        );
      },
      run: async () => {
        const index = createStructureIndex(store.editSnapshot.structure);
        const structureNodeIds = selectedStructureNodeIds();
        const locked = structureNodeIds.every((id) => index.structureById.get(id)?.locked);
        await store.executeSpaceEdit({
          kind: "structure.set-lock",
          rootIds: structureNodeIds,
          locked: !locked,
        });
      },
    }),
  ]);
}

function command(
  id: string,
  title: string,
  input: Omit<AppCommand, "id" | "title" | "category"> & { category?: CommandCategory },
): AppCommand {
  const { category = "canvas", ...commandInput } = input;
  return { id, title, category, ...commandInput };
}
