import { invoke } from "@tauri-apps/api/core";

import type { CanvasPersistenceMutationRequest } from "../contract/canvas-editor";
export function commitCanvasGesture(mutation: CanvasPersistenceMutationRequest, spaceId: string) {
  return invoke<{ revision: number }>("plugin:spaces|commit_canvas_gesture", {
    request: { spaceId, baseRevision: mutation.baseRevision, document: mutation.document },
  });
}
