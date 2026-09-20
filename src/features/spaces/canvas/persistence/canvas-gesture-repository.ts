import { invoke } from "@tauri-apps/api/core";

import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { SpaceEditorDocument } from "../contract/canvas-document";
import type { CanvasPersistenceMutationRequest } from "../contract/canvas-editor";

export interface CanvasGesturePersistenceRequest {
  spaceId: string;
  changes: readonly { id: string; data: SpaceNodePayload }[];
  baseRevision: number;
  document: SpaceEditorDocument;
}

export function commitCanvasGesture(
  mutation: CanvasPersistenceMutationRequest,
  spaceId: string,
  changes: CanvasGesturePersistenceRequest["changes"],
) {
  const request: CanvasGesturePersistenceRequest = {
    spaceId,
    baseRevision: mutation.baseRevision,
    document: mutation.document,
    changes,
  };
  return invoke<{ revision: number }>("plugin:spaces|commit_canvas_gesture", { request });
}
