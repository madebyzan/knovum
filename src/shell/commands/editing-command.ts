import { getAppCommand, runAppCommand } from "../../foundation/commands/run-app-command";
import { performApplicationMenuEditing } from "../../foundation/commands/text-editing";
import { isEditableTarget } from "../../foundation/dom/editable-target";
import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";

// The macOS Application Menu owns every standard editing chord, so each one arrives here as an
// intent instead of a key press. Clipboard and selection belong to the focused text control when
// there is one, because only the platform can edit a native text field. Undo belongs to whichever
// surface keeps the history, so it always resolves to that surface's own command.
export type EditingIntent = EditingCommand | "undo" | "redo";

export const EDITING_INTENTS: readonly EditingIntent[] = [
  "undo",
  "redo",
  "cut",
  "copy",
  "paste",
  "select-all",
];

const SURFACE_COMMANDS: Record<EditingIntent, readonly string[]> = {
  undo: ["document.undo", "spaces.undo"],
  redo: ["document.redo", "spaces.redo"],
  cut: ["spaces.cut"],
  copy: ["spaces.copy"],
  paste: ["spaces.paste"],
  "select-all": ["spaces.select-all"],
};

function textControlAction(intent: EditingIntent): EditingCommand | undefined {
  return intent === "undo" || intent === "redo" ? undefined : intent;
}

export function runEditingCommand(intent: EditingIntent): void | Promise<void> {
  const action = textControlAction(intent);
  if (action && isEditableTarget(document.activeElement)) {
    return performApplicationMenuEditing(action);
  }
  for (const commandId of SURFACE_COMMANDS[intent]) {
    const command = getAppCommand(commandId);
    if (command) {
      runAppCommand(command);
      return;
    }
  }
}
