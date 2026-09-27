import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import { parseSpaceEditSnapshot, type SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { invokeConfirmedMutation } from "../structure/confirmed-mutation";
import type { SpaceClipboardPayload } from "./clipboard-envelope";

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
  const clipboardCommit = await invokeConfirmedMutation<ImportStructureClipboardResponse>(
    "plugin:spaces|import_structure_clipboard",
    { request },
    request.mutationId,
  );
  return {
    ...clipboardCommit,
    snapshot: parseSpaceEditSnapshot(clipboardCommit.snapshot),
  } satisfies {
    snapshot: SpaceEditSnapshot;
    nodes: SpaceCanvasNode[];
    relations: SpaceRelation[];
    selectionSuggestion: string[];
  };
}
