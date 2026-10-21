import { platform } from "@tauri-apps/plugin-os";

import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";
import { invokeCommand } from "../ipc/invoke-command";

export const EDITING_COMMANDS: readonly EditingCommand[] = [
  "undo",
  "redo",
  "cut",
  "copy",
  "paste",
  "select-all",
];

/**
 * The standard editing chords and the intent each one carries. They are the one set of keys whose
 * owner depends on what the user is editing, so only the editing responder chain may bind them.
 */
export const EDITING_CHORDS: Readonly<Record<string, EditingCommand>> = {
  "$mod+Z": "undo",
  "$mod+Shift+Z": "redo",
  "$mod+Y": "redo",
  "$mod+X": "cut",
  "$mod+C": "copy",
  "$mod+V": "paste",
  "$mod+A": "select-all",
};

/**
 * macOS resolves the editing chords through the Application Menu, so the WebView never receives
 * them; the platform acts only when AppKit sends the action to the first responder.
 */
export function editingChordsReachWebView(): boolean {
  return platform() !== "macos";
}

export function performApplicationMenuEditing(command: EditingCommand): void | Promise<void> {
  if (editingChordsReachWebView()) return;
  return invokeCommand<boolean>("plugin:desktop|execute_editing_command", { command }).then(() => undefined);
}
