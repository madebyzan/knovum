import { getAppCommand, runAppCommand } from "../../foundation/commands/run-app-command";
import { performApplicationMenuEditing } from "../../foundation/commands/text-editing";
import { isEditableTarget, isTextControlTarget } from "../../foundation/dom/editable-target";
import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";

// The macOS Application Menu owns every standard editing chord, so each one arrives here as an
// intent instead of a key press. This router answers one question: who owns the text being edited?

export const EDITING_COMMANDS: readonly EditingCommand[] = [
  "undo",
  "redo",
  "cut",
  "copy",
  "paste",
  "select-all",
];

/**
 * The surface command that performs the intent when the platform does not. A document keeps its own
 * edit history, so it answers before the canvas does.
 */
const SURFACE_COMMAND_IDS_BY_EDITING_COMMAND: Record<EditingCommand, readonly string[]> = {
  undo: ["document.undo", "spaces.undo"],
  redo: ["document.redo", "spaces.redo"],
  cut: ["spaces.cut"],
  copy: ["spaces.copy"],
  paste: ["spaces.paste"],
  "select-all": ["spaces.select-all"],
};

/**
 * Clipboard and selection belong to any focused editable target, because only the platform can put
 * the system clipboard into it and a custom editor still receives the resulting DOM events. Edit
 * history is narrower: a native text control owns its own, while a rich-text surface keeps history
 * in its own stack and must be undone through its command.
 */
function platformOwns(intent: EditingCommand, target: EventTarget | null) {
  return intent === "undo" || intent === "redo" ? isTextControlTarget(target) : isEditableTarget(target);
}

export function runEditingCommand(intent: EditingCommand): void | Promise<void> {
  if (platformOwns(intent, document.activeElement)) return performApplicationMenuEditing(intent);
  for (const commandId of SURFACE_COMMAND_IDS_BY_EDITING_COMMAND[intent]) {
    const command = getAppCommand(commandId);
    if (command) {
      runAppCommand(command);
      return;
    }
  }
}
