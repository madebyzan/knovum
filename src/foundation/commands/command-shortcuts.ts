import { platform } from "@tauri-apps/plugin-os";
import { tinykeys } from "tinykeys";

import type { AppCommand } from "../../shared/contracts/app-command";
import { isEditableTarget } from "../dom/editable-target";
import { isInteractiveTarget } from "../dom/interactive-target";
import { handleEditingChord } from "./editing-responder";
import { runAppCommand } from "./run-app-command";
import { EDITING_CHORDS, editingChordsReachWebView } from "./text-editing";

function resolveKeybindings(command: AppCommand, effective?: string): readonly string[] {
  if (effective && effective === command.shortcut && command.keybindings) {
    return command.keybindings;
  }
  return effective ? [effective] : [];
}

function assertCommandChordsUnclaimed(command: AppCommand, bindings: readonly string[]): void {
  for (const keybinding of bindings) {
    if (keybinding in EDITING_CHORDS) {
      throw new Error(`Command ${command.id} binds the editing chord ${keybinding}; declare editingIntent.`);
    }
  }
}

function assertEditingChordsUnclaimed(
  commands: readonly AppCommand[],
  resolveShortcut?: (commandId: string, defaultShortcut?: string) => string | undefined,
) {
  const claimedIntents = new Set<string>();
  for (const command of commands) {
    if (command.editingIntent) {
      if (claimedIntents.has(command.editingIntent)) {
        throw new Error(`Command ${command.id} is a second responder for "${command.editingIntent}".`);
      }
      claimedIntents.add(command.editingIntent);
      continue;
    }
    const effective = resolveShortcut?.(command.id, command.shortcut) ?? command.shortcut;
    assertCommandChordsUnclaimed(command, resolveKeybindings(command, effective));
  }
}

function registerCommandHandlers(
  command: AppCommand,
  effective: string,
  handlers: Record<string, (event: KeyboardEvent) => void>,
) {
  for (const keybinding of resolveKeybindings(command, effective)) {
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

function shouldSkipCommandBinding(
  command: AppCommand,
  effective: string | undefined,
  isCustomized: boolean,
  appMenuOwnsShortcut: boolean,
): boolean {
  if (!effective) return true;
  if (command.editingIntent && !isCustomized) return true;
  if (appMenuOwnsShortcut && command.appMenuShortcut && !isCustomized) return true;
  return false;
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
    const isCustomized = Boolean(resolveShortcut && effective !== command.shortcut);
    if (shouldSkipCommandBinding(command, effective, isCustomized, appMenuOwnsShortcut)) {
      continue;
    }
    registerCommandHandlers(command, effective!, handlerByShortcut);
  }

  return tinykeys(globalThis.window, handlerByShortcut, {
    capture: true,
    ignore: (event) => event.isComposing,
  });
}
