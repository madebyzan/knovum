import { platform } from "@tauri-apps/plugin-os";
import { tinykeys } from "tinykeys";

import type { AppCommand } from "../../shared/contracts/app-command";
import { isEditableTarget } from "../dom/editable-target";
import { isInteractiveTarget } from "../dom/interactive-target";
import { runAppCommand } from "./run-app-command";
import { TEXT_EDITING_CHORDS } from "./text-editing";

export function bindCommandShortcuts(commands: readonly AppCommand[]): () => void {
  const handlerByShortcut: Record<string, (event: KeyboardEvent) => void> = {};
  const appMenuOwnsShortcut = platform() === "macos";

  for (const command of commands) {
    if (!command.shortcut || (appMenuOwnsShortcut && command.appMenuShortcut)) continue;
    for (const keybinding of command.keybindings ?? [command.shortcut]) {
      // A modifier chord belongs to the application, but a bare key is the class the platform hands
      // to the focused control, so Space on a button and the arrows on a slider stay theirs.
      const bareKey = !keybinding.includes("+");
      // Text editing stays with the text being edited: a command may keep working while the user
      // types, but never by taking the chord that edits what they are typing.
      const platformOwnsWhileTyping = TEXT_EDITING_CHORDS.includes(keybinding);

      handlerByShortcut[keybinding] = (event) => {
        const editable = isEditableTarget(event.target);
        if (editable && (platformOwnsWhileTyping || !command.allowInEditable)) return;
        if (bareKey && isInteractiveTarget(event.target)) return;
        event.preventDefault();
        runAppCommand(command);
      };
    }
  }

  // tinykeys' default filter also drops every press from an editable target before the per-command
  // `isEditableTarget` guard above can allow it, which would silently void `allowInEditable`.
  // Autorepeat and IME composition are still somebody else's key press.
  return tinykeys(globalThis.window, handlerByShortcut, {
    capture: true,
    ignore: (event) => event.repeat || event.isComposing,
  });
}
