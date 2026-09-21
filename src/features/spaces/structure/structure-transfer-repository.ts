import { invokeConfirmedMutation } from "../organization/confirmed-mutation";

export interface TransferStructureSelectionRequest {
  sourceSpaceId: string;
  mutationId: string;
  targetSpaceId: string;
  rootStructureNodeIds: readonly string[];
  targetParentId: string | null;
  targetOrder: number;
  expectedSourceEditRevision: number;
  expectedTargetEditRevision: number;
}

export interface TransferStructureSelectionResponse {
  movedNodeIds: string[];
  movedStructureNodeIds: string[];
  sourceEditRevision: number;
  targetEditRevision: number;
}

export function moveStructureSelectionBetweenSpaces(
  request: TransferStructureSelectionRequest,
): Promise<TransferStructureSelectionResponse> {
  return invokeConfirmedMutation<TransferStructureSelectionResponse>(
    "plugin:spaces|move_structure_selection",
    { request },
    request.mutationId,
  );
}
