import type { CanvasJsonImportRequest } from "../canvas-exchange/json-canvas-import-contract";
import type { SpaceEditorDocument } from "../canvas/contract/canvas-document";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export const emptyCanvasDocument: SpaceEditorDocument = {
  version: 1,
  viewport: { x: 0, y: 0, zoom: 1 },
  nodes: [],
};

export function canvasNote(id: string): SpaceCanvasNode {
  return { id, type: "note", data: createSpaceNotePayload(id, ""), initialPosition: { x: 0, y: 0 } };
}

export function jsonCanvasTextImportRequest(): CanvasJsonImportRequest {
  return {
    sourceDirectory: "/unused",
    plan: {
      nodes: [
        {
          sourceId: "text",
          frame: { x: 0, y: 0, width: 320, height: 180, rotation: 0 },
          zIndex: 0,
          content: { type: "text", text: "Evidence" },
        },
      ],
      edges: [],
    },
    baseRevision: 0,
    document: emptyCanvasDocument,
  };
}

export function deferred<T>() {
  let resolvePromise: ((value: T) => void) | undefined;
  const promise = new Promise<T>((resolve) => {
    resolvePromise = resolve;
  });
  return {
    promise,
    resolve(value: T) {
      if (!resolvePromise) throw new Error("Deferred promise is not initialized");
      resolvePromise(value);
    },
  };
}
