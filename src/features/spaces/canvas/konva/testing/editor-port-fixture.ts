import type { SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasPersistenceHistory, CanvasSurfaceEvents } from "../../contract/canvas-editor";

export const editorPortDocument: SpaceEditorDocument = {
  version: 1,
  viewport: { x: 0, y: 0, zoom: 1 },
  nodes: [
    {
      nodeId: "selected",
      frame: { x: 120, y: 240, width: 300, height: 220, rotation: 0 },
      sizeAuthority: "frame",
      zIndex: 0,
    },
  ],
};

export const editorPortHistory: CanvasPersistenceHistory = {
  undo: async () => ({ revision: 2 }),
  redo: async () => ({ revision: 3 }),
};

export function editorPortHostEvents(): CanvasSurfaceEvents {
  return {
    onCommitGesture: async (request) => ({
      document: request.document,
      revision: 1,
      history: editorPortHistory,
    }),
    onCopyNodes: async () => {},
    onCreateRelation: async () => {},
    onCreateTextNode: async () => "created",
    onCutSelection: async (request) => ({
      document: request.document,
      revision: 1,
      history: editorPortHistory,
    }),
    onPasteNodes: async () => true,
    onDeleteSelection: async (request) => ({
      document: request.document,
      revision: 1,
      history: editorPortHistory,
    }),
    onDuplicateNodes: async () => {},
    onInsertText: async () => {},
    onInsertUrl: async () => {},
    onImportFiles: async () => [],
    onNodeActivate: async () => {},
    onError() {},
    onHistoryEntry() {},
    onHistoryTransition: async (document, transition) => ({
      ...(await transition({ baseRevision: 1, document })),
      document,
    }),
    onHistoryTransaction: async (action) => action(),
    onMutationPendingChange() {},
    onDirectManipulationChange() {},
    onReady() {},
    onSelectionChange() {},
    onViewportChange() {},
  };
}

export function documentElement() {
  return globalThis.document.createElement("div");
}
