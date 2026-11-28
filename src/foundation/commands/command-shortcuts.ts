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
  resolveCommand: (id: string) => AppCommand | undefined,
) {
  for (const keybinding of resolveKeybindings(command, effective)) {
    const bareKey = !keybinding.includes("+");
    const commandId = command.id;
    handlers[keybinding] = (event) => {
      const current = resolveCommand(commandId);
      if (!current || (event.repeat && !current.allowRepeat)) return;
      if (isEditableTarget(event.target) && !current.allowInEditable) return;
      if (bareKey && isInteractiveTarget(event.target)) return;
      event.preventDefault();
      runAppCommand(current);
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

/** Stable fingerprint of the active shortcut bindings so unchanged command lists skip rebinding tinykeys. */
export function commandShortcutSignature(
  commands: readonly AppCommand[],
  resolveShortcut?: (commandId: string, defaultShortcut?: string) => string | undefined,
): string {
  const appMenuOwnsShortcut = platform() === "macos";
  const parts: string[] = [];
  for (const command of commands) {
    const effective = resolveShortcut?.(command.id, command.shortcut) ?? command.shortcut;
    const isCustomized = Boolean(resolveShortcut && effective !== command.shortcut);
    if (shouldSkipCommandBinding(command, effective, isCustomized, appMenuOwnsShortcut)) {
      if (command.editingIntent) parts.push(`${command.id}=intent:${command.editingIntent}`);
      continue;
    }
    parts.push(
      `${command.id}=${resolveKeybindings(command, effective!).join(",")}:${command.allowInEditable ? "1" : "0"}`,
    );
  }
  return parts.join("|");
}

export function bindCommandShortcuts(
  commandsSource: readonly AppCommand[] | (() => readonly AppCommand[]),
  resolveShortcut?: (commandId: string, defaultShortcut?: string) => string | undefined,
): () => void {
  const getCommands = typeof commandsSource === "function" ? commandsSource : () => commandsSource;
  const commands = getCommands();
  assertEditingChordsUnclaimed(commands, resolveShortcut);
  const handlerByShortcut: Record<string, (event: KeyboardEvent) => void> = {};
  const appMenuOwnsShortcut = platform() === "macos";
  const resolveCommand = (id: string) => getCommands().find((candidate) => candidate.id === id);

  if (editingChordsReachWebView()) {
    for (const [chord, intent] of Object.entries(EDITING_CHORDS)) {
      handlerByShortcut[chord] = (event) => handleEditingChord(intent, getCommands(), event);
    }
  }
  for (const command of commands) {
    const effective = resolveShortcut?.(command.id, command.shortcut) ?? command.shortcut;
    const isCustomized = Boolean(resolveShortcut && effective !== command.shortcut);
    if (shouldSkipCommandBinding(command, effective, isCustomized, appMenuOwnsShortcut)) {
      continue;
    }
    registerCommandHandlers(command, effective!, handlerByShortcut, resolveCommand);
  }

  return tinykeys(globalThis.window, handlerByShortcut, {
    capture: true,
    ignore: (event) => event.isComposing,
  });
}
