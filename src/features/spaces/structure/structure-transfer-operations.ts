import type { TransferSelectionRequest } from "../contracts/generated/TransferSelectionRequest";
import type { TransferSelectionResponse } from "../contracts/generated/TransferSelectionResponse";
import { invokeConfirmedMutation } from "./confirmed-mutation";

export function moveStructureSelectionBetweenSpaces(
  request: TransferSelectionRequest,
): Promise<TransferSelectionResponse> {
  return invokeConfirmedMutation<TransferSelectionResponse>(
    "plugin:spaces|move_structure_selection",
    { request },
    request.mutationId,
  );
}
