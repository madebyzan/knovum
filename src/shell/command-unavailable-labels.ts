import type { CommandUnavailableReason } from "../shared/contracts/app-command";

export const commandUnavailableLabelKeys = {
  nothing_to_undo: "nothingToUndo",
  nothing_to_redo: "nothingToRedo",
  no_selection: "noSelection",
  active_node_operation: "activeNodeOperation",
  not_enough_scenes: "notEnoughScenes",
  selection_not_eligible: "selectionNotEligible",
} as const satisfies Record<CommandUnavailableReason, string>;
