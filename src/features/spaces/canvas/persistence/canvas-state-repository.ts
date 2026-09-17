import { invoke } from "@tauri-apps/api/core";

import { parseCanvasDocument } from "../contract/canvas-document";
import type {
  CanvasPersistencePort,
  SaveCanvasStateRequest,
  StoredCanvasState,
} from "../contract/canvas-persistence";

export const canvasStateRepository: CanvasPersistencePort = {
  async load(spaceId: string) {
    const state = await invoke<StoredCanvasState | null>("plugin:spaces|load_canvas_state", { spaceId });
    return state ? { ...state, document: parseCanvasDocument(state.document) } : null;
  },
  save(request: SaveCanvasStateRequest) {
    return invoke<number>("plugin:spaces|save_canvas_state", { request });
  },
};
