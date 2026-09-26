import type { TransferSelectionRequest as TransferStructureSelectionRequest } from "../contracts/generated/TransferSelectionRequest";
import type { TransferSelectionResponse as TransferStructureSelectionResponse } from "../contracts/generated/TransferSelectionResponse";
export type { TransferSelectionRequest as TransferStructureSelectionRequest } from "../contracts/generated/TransferSelectionRequest";
export type { TransferSelectionResponse as TransferStructureSelectionResponse } from "../contracts/generated/TransferSelectionResponse";
import { invokeConfirmedMutation } from "../organization/confirmed-mutation";

export function moveStructureSelectionBetweenSpaces(
  request: TransferStructureSelectionRequest,
): Promise<TransferStructureSelectionResponse> {
  return invokeConfirmedMutation<TransferStructureSelectionResponse>(
    "plugin:spaces|move_structure_selection",
    { request },
    request.mutationId,
  );
}
