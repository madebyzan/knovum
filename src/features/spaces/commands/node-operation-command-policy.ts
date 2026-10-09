import type { AppCommand } from "../../../shared/contracts/app-command";
import type { NodeOperationEffect } from "../canvas/use-node-operation-gate";
import { guardNodeOperationCommand } from "./guard-node-operation-command";

type Allows = (effect: NodeOperationEffect) => boolean;
const readOnlySelectionCommands = new Set(["spaces.focus-outline", "spaces.find", "spaces.find-all"]);
const readOnlyNavigationCommands = new Set([
  "spaces.cancel-canvas-interaction",
  "spaces.zoom-in",
  "spaces.zoom-out",
  "spaces.zoom-reset",
  "spaces.zoom-to-fit",
  "spaces.zoom-to-selection",
  "spaces.ask",
  "spaces.focus-selection",
]);

export function guardSelectionCommands(commands: readonly AppCommand[], allows: Allows) {
  return commands.map((command) =>
    readOnlySelectionCommands.has(command.id)
      ? command
      : guardNodeOperationCommand(command, "structure", allows),
  );
}

export function guardNavigationCommands(commands: readonly AppCommand[], allows: Allows) {
  return commands.map((command) =>
    readOnlyNavigationCommands.has(command.id)
      ? command
      : guardNodeOperationCommand(command, "structure", allows),
  );
}

export function guardAllCommands(
  commands: readonly AppCommand[],
  effect: NodeOperationEffect,
  allows: Allows,
) {
  return commands.map((command) => guardNodeOperationCommand(command, effect, allows));
}

export function guardCanvasExchangeCommands(commands: readonly AppCommand[], allows: Allows) {
  return commands.map((command) =>
    guardNodeOperationCommand(
      command,
      command.id === "spaces.import-json-canvas" ? "create" : "export",
      allows,
    ),
  );
}
