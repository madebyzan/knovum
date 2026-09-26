import type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";
export type { ExecuteSpaceEditRequest } from "../contracts/generated/ExecuteSpaceEditRequest";
export type { SpaceEditReceipt } from "../contracts/generated/SpaceEditReceipt";
export type { FieldUpdate } from "../contracts/generated/FieldUpdate";
export type { SpacePoint } from "../contracts/generated/SpacePoint";
export type { SpaceRect } from "../contracts/generated/SpaceRect";
export type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";

export function spaceEditCommandRecordsHistory(command: SpaceEditCommand) {
  return command.kind !== "content.update-heights";
}
