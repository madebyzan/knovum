import { canRunAction } from "../../../shared/contracts/action-availability";
import type { AppCommand } from "../../../shared/contracts/app-command";
import type { NodeOperationEffect } from "../canvas/node-operation-gate";

export function guardNodeOperationCommand(
  command: AppCommand,
  effect: NodeOperationEffect,
  allows: (effect: NodeOperationEffect) => boolean,
): AppCommand {
  return {
    ...command,
    availability: () =>
      allows(effect)
        ? (command.availability?.() ?? { state: "ready" as const })
        : { state: "blocked" as const, reason: "active_node_operation" as const },
    run: () => {
      if (allows(effect) && (!command.availability || canRunAction(command.availability()))) {
        return command.run();
      }
    },
  };
}
