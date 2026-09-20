import type { SpaceEditorDocument } from "./canvas-document";

export interface StoredCanvasState {
  document: SpaceEditorDocument;
  revision: number;
}

export interface SaveCanvasStateRequest {
  spaceId: string;
  baseRevision: number;
  document: SpaceEditorDocument;
}

export interface CanvasPersistencePort {
  load(spaceId: string): Promise<StoredCanvasState | null>;
  save(request: SaveCanvasStateRequest): Promise<number>;
}
