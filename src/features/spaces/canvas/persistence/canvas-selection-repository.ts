import { invoke } from "@tauri-apps/api/core";

import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { SpaceEditorDocument } from "../contract/canvas-document";
import type { CanvasDeleteSelectionRequest } from "../contract/canvas-editor";

interface CanvasSelectionPersistenceRequest extends CanvasDeleteSelectionRequest {
  spaceId: string;
  survivorChanges: readonly { id: string; data: SpaceNodePayload }[];
}

export function deleteCanvasSelection(request: CanvasSelectionPersistenceRequest) {
  return invoke<{ revision: number }>("plugin:spaces|delete_canvas_selection", { request });
}

export function restoreCanvasSelection(request: CanvasSelectionPersistenceRequest) {
  return invoke<{ revision: number }>("plugin:spaces|restore_canvas_selection", { request });
}

export function setImportedCanvasNodesDeleted(request: {
  spaceId: string;
  nodeIds: readonly string[];
  relationIds: readonly string[];
  deleted: boolean;
  baseRevision: number;
  document: SpaceEditorDocument;
}) {
  return invoke<{ revision: number }>("plugin:spaces|set_imported_canvas_nodes_deleted", { request });
}
