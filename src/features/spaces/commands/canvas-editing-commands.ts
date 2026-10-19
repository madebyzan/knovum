import { info as logInfo } from "@tauri-apps/plugin-log";
import { platform } from "@tauri-apps/plugin-os";

import { availableWhen } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";
import type { CanvasEditorPort } from "../canvas/contracts/canvas-editor-port";
import { runEditorMutation } from "../workspace/editor-protection-feedback";

interface CanvasEditingCommandLabels {
  copy: string;
  cut: string;
  deleteSelection: string;
  duplicate: string;
  paste: string;
  redo: string;
  selectAll: string;
  undo: string;
}

interface CanvasEditingCommandOptions {
  canRedo: () => boolean;
  canUndo: () => boolean;
  editor: () =>
    | Pick<
        CanvasEditorPort,
        | "copySelection"
        | "cutSelection"
        | "deleteSelection"
        | "duplicateSelection"
        | "pasteSelection"
        | "selectAll"
      >
    | undefined;
  hasNodeSelection: () => boolean;
  hasSelection: () => boolean;
  labels: CanvasEditingCommandLabels;
  redo: () => Promise<void>;
  undo: () => Promise<void>;
}

export function createCanvasEditingCommands(options: CanvasEditingCommandOptions): AppCommand[] {
  const editor = options.editor;
  return [
    {
      id: "spaces.undo",
      title: options.labels.undo,
      shortcut: "$mod+Z",
      appMenuShortcut: true,
      category: "editing",
      availability: () => availableWhen(options.canUndo(), "nothing_to_undo"),
      run: () => runEditorMutation(options.undo),
    },
    {
      id: "spaces.redo",
      title: options.labels.redo,
      shortcut: "$mod+Shift+Z",
      appMenuShortcut: true,
      category: "editing",
      availability: () => availableWhen(options.canRedo(), "nothing_to_redo"),
      run: () => runEditorMutation(options.redo),
    },
    {
      id: "spaces.copy",
      title: options.labels.copy,
      shortcut: "$mod+C",
      appMenuShortcut: true,
      category: "editing",
      availability: () => availableWhen(options.hasNodeSelection(), "no_selection"),
      run: () => editor()?.copySelection(),
    },
    {
      id: "spaces.cut",
      title: options.labels.cut,
      shortcut: "$mod+X",
      appMenuShortcut: true,
      category: "editing",
      availability: () => availableWhen(options.hasNodeSelection(), "no_selection"),
      run: () => runEditorMutation(() => editor()?.cutSelection()),
    },
    {
      id: "spaces.paste",
      title: options.labels.paste,
      shortcut: "$mod+V",
      appMenuShortcut: true,
      category: "editing",
      run: async () => {
        const canvasEditor = editor();
        await logInfo(`[spaces.paste] command invoked; editor=${canvasEditor ? "active" : "missing"}`);
        if (!canvasEditor) return;
        const pasted = await canvasEditor.pasteSelection();
        await logInfo(`[spaces.paste] command completed; pasted=${pasted}`);
      },
    },
    {
      id: "spaces.duplicate",
      title: options.labels.duplicate,
      shortcut: "$mod+D",
      category: "editing",
      availability: () => availableWhen(options.hasNodeSelection(), "no_selection"),
      run: () => editor()?.duplicateSelection(),
    },
    {
      id: "spaces.delete-selection",
      title: options.labels.deleteSelection,
      shortcut: platform() === "macos" ? "Backspace" : "Delete",
      category: "editing",
      availability: () => availableWhen(options.hasSelection(), "no_selection"),
      run: () => runEditorMutation(() => editor()?.deleteSelection()),
    },
    {
      id: "spaces.select-all",
      title: options.labels.selectAll,
      shortcut: "$mod+A",
      appMenuShortcut: true,
      category: "canvas",
      run: () => editor()?.selectAll(),
    },
  ];
}
