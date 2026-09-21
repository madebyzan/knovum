import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import { invokeConfirmedMutation } from "../organization/confirmed-mutation";
import type { SpaceClipboardPayload } from "./payload-types";

export interface ImportStructureClipboardRequest {
  spaceId: string;
  mutationId: string;
  expectedEditRevision: number;
  targetParentId: string | null;
  anchor: { x: number; y: number };
  payload: SpaceClipboardPayload;
}

interface ImportStructureClipboardResponse {
  snapshot: unknown;
  nodes: SpaceCanvasNode[];
  relations: SpaceRelation[];
  selectionSuggestion: string[];
}

export async function importStructureClipboard(request: ImportStructureClipboardRequest) {
  const result = await invokeConfirmedMutation<ImportStructureClipboardResponse>(
    "plugin:spaces|import_structure_clipboard",
    { request },
    request.mutationId,
  );
  return {
    ...result,
    snapshot: parseSpaceEditSnapshot(result.snapshot),
  } satisfies {
    snapshot: SpaceEditSnapshot;
    nodes: SpaceCanvasNode[];
    relations: SpaceRelation[];
    selectionSuggestion: string[];
  };
}
