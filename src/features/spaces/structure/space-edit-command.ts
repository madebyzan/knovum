import type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";

export function spaceEditCommandRecordsHistory(command: SpaceEditCommand) {
  return command.kind !== "content.update-heights";
}
