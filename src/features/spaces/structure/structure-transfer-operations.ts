import type { SpaceEditorResource } from "../contracts/generated/SpaceEditorResource";
import type { TransferSelectionRequest } from "../contracts/generated/TransferSelectionRequest";
import type { TransferSelectionResponse } from "../contracts/generated/TransferSelectionResponse";
import { invokeConfirmedMutation } from "./confirmed-mutation";

export function moveStructureSelectionBetweenSpaces(
  request: TransferSelectionRequest,
  editorResources: readonly SpaceEditorResource[],
): Promise<TransferSelectionResponse> {
  return invokeConfirmedMutation<TransferSelectionResponse>(
    "plugin:spaces|move_structure_selection",
    { request, editorResources },
    request.mutationId,
  );
}
