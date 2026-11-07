import type { AppCommand } from "../../shared/contracts/app-command";
import { normalizeChord } from "./shortcut-chord";

export function findShortcutConflict(
  targetCommandId: string,
  chord: string,
  commands: readonly AppCommand[],
  customShortcuts: Readonly<Record<string, string>>,
): AppCommand | undefined {
  const normalizedTarget = normalizeChord(chord);
  if (!normalizedTarget) return undefined;

  for (const cmd of commands) {
    if (cmd.id === targetCommandId) continue;
    const effective = customShortcuts[cmd.id] ?? cmd.shortcut;
    if (!effective) continue;
    if (normalizeChord(effective) === normalizedTarget) {
      return cmd;
    }
  }
  return undefined;
}
