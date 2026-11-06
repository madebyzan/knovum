import { platform } from "@tauri-apps/plugin-os";
import { tinykeys } from "tinykeys";

import type { AppCommand } from "../../shared/contracts/app-command";
import { isEditableTarget } from "../dom/editable-target";
import { isInteractiveTarget } from "../dom/interactive-target";
import { handleEditingChord } from "./editing-responder";
import { runAppCommand } from "./run-app-command";
import { EDITING_CHORDS, editingChordsReachWebView } from "./text-editing";

/**
 * An editing command answers an intent, never a key: which command a chord reaches depends on what
 * the user is editing, so a command that bound the chord itself would take it from the text.
 */
function assertEditingChordsUnclaimed(
  commands: readonly AppCommand[],
  resolveShortcut?: (commandId: string, defaultShortcut?: string) => string | undefined,
) {
  const claimedIntents = new Set<string>();
  for (const command of commands) {
    if (command.editingIntent) {
      if (claimedIntents.has(command.editingIntent))
        throw new Error(`Command ${command.id} is a second responder for "${command.editingIntent}".`);
      claimedIntents.add(command.editingIntent);
      continue;
    }
    const effective = resolveShortcut?.(command.id, command.shortcut) ?? command.shortcut;
    for (const keybinding of command.keybindings ?? (effective ? [effective] : [])) {
      if (keybinding in EDITING_CHORDS)
        throw new Error(
          `Command ${command.id} binds the editing chord ${keybinding}; declare editingIntent.`,
        );
    }
  }
}

function registerCommandHandlers(
  command: AppCommand,
  effective: string,
  handlers: Record<string, (event: KeyboardEvent) => void>,
) {
  for (const keybinding of command.keybindings ?? [effective]) {
    const bareKey = !keybinding.includes("+");
    handlers[keybinding] = (event) => {
      if (event.repeat) return;
      if (isEditableTarget(event.target) && !command.allowInEditable) return;
      if (bareKey && isInteractiveTarget(event.target)) return;
      event.preventDefault();
      runAppCommand(command);
    };
  }
}

export function bindCommandShortcuts(
  commands: readonly AppCommand[],
  resolveShortcut?: (commandId: string, defaultShortcut?: string) => string | undefined,
): () => void {
  assertEditingChordsUnclaimed(commands, resolveShortcut);
  const handlerByShortcut: Record<string, (event: KeyboardEvent) => void> = {};
  const appMenuOwnsShortcut = platform() === "macos";

  if (editingChordsReachWebView()) {
    for (const [chord, intent] of Object.entries(EDITING_CHORDS)) {
      handlerByShortcut[chord] = (event) => handleEditingChord(intent, commands, event);
    }
  }
  for (const command of commands) {
    const effective = resolveShortcut?.(command.id, command.shortcut) ?? command.shortcut;
    if (!effective || command.editingIntent || (appMenuOwnsShortcut && command.appMenuShortcut)) {
      continue;
    }
    registerCommandHandlers(command, effective, handlerByShortcut);
  }

  // tinykeys' default filter also drops every press from an editable target before the per-command
  // `isEditableTarget` guard above can allow it, which would silently void `allowInEditable`.
  // IME composition is still somebody else's key press.
  return tinykeys(globalThis.window, handlerByShortcut, {
    capture: true,
    ignore: (event) => event.isComposing,
  });
}
