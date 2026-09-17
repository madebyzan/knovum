import type { CanvasDocumentV1 } from "./canvas-document";

export interface StoredCanvasState {
  document: CanvasDocumentV1;
  revision: number;
}

export interface SaveCanvasStateRequest {
  spaceId: string;
  baseRevision: number;
  document: CanvasDocumentV1;
}

export interface CanvasPersistencePort {
  load(spaceId: string): Promise<StoredCanvasState | null>;
  save(request: SaveCanvasStateRequest): Promise<number>;
}
