import { runAppCommandById } from "../../foundation/commands/run-app-command";
import { performApplicationMenuEditing } from "../../foundation/commands/text-editing";
import { isEditableTarget } from "../../foundation/dom/editable-target";
import type { EditingCommand } from "../../shared/contracts/generated/EditingCommand";

export function runEditingCommand(command: EditingCommand): void | Promise<void> {
  const focused = document.activeElement;
  if (isEditableTarget(focused)) {
    return performApplicationMenuEditing(command);
  }
  runAppCommandById(`spaces.${command}`);
}
