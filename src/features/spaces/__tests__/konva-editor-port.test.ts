import { describe, expect, it, vi } from "vitest";

import type { CanvasSurfaceEvents } from "../canvas/contract/canvas-editor";
import { createKonvaEditorPort } from "../canvas/konva/create-canvas-editor-port";
import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
import {
  documentElement,
  editorPortDocument as document,
  editorPortHistory as history,
  editorPortHostEvents as hostEvents,
} from "./konva-editor-port-fixture";

describe("Konva editor port", () => {
  it("routes clipboard operations through product ports and uses the last canvas pointer", async () => {
    const copy = vi.fn<CanvasSurfaceEvents["onCopyNodes"]>(async () => {});
    const cut = vi.fn<CanvasSurfaceEvents["onCutSelection"]>(async (request) => ({
      document: request.document,
      revision: 1,
      history,
    }));
    const paste = vi.fn<CanvasSurfaceEvents["onPasteNodes"]>(async () => true);
    const historyEntry = vi.fn<CanvasSurfaceEvents["onHistoryEntry"]>();
    const model = new CanvasDocumentModel(document, {
      documentChanged() {},
      historyEntry() {},
      viewportChanged() {},
    });
    const port = createKonvaEditorPort({
      cancelInteraction() {},
      runFrameMutation: async (mutation) => {
        await mutation();
      },
      host: () => documentElement(),
      layouts: () => document.nodes,
      model,
      nodeById: () => undefined,
      nudgeSelection: async () => {},
      nodes: () => [],
      relations: () => [],
      pastePoint: () => ({ x: 55, y: 89 }),
      relationIds: () => new Set(),
      relationBounds: () => new Map(),
      selectableNodeIds: () => ["selected"],
      selectedNodeIds: () => new Set(["selected"]),
      setSelection() {},
      setViewport() {},
      size: () => ({ width: 800, height: 600 }),
      viewport: () => document.viewport,
      onCommitGesture: async (request) => ({ document: request.document, revision: 1, history }),
      onCopyNodes: copy,
      onCreateRelation: async () => {},
      onCreateTextNode: async () => "created",
      onCutSelection: cut,
      onPasteNodes: paste,
      onDeleteSelection: async (request) => ({ document: request.document, revision: 1, history }),
      onDuplicateNodes: async () => {},
      onInsertText: async () => {},
      onInsertUrl: async () => {},
      onImportFiles: async () => [],
      onNodeActivate: async () => {},
      onError() {},
      onHistoryEntry: historyEntry,
      onHistoryTransition: async (nextDocument, transition) => ({
        ...(await transition({ baseRevision: 1, document: nextDocument })),
        document: nextDocument,
      }),
      onHistoryTransaction: async (action) => action(),
      onMutationPendingChange() {},
      onDirectManipulationChange() {},
      onReady() {},
      onSelectionChange() {},
      onViewportChange() {},
    });

    await port.copySelection();
    await port.cutSelection();
    await port.pasteSelection();
    expect(copy).toHaveBeenCalledWith([{ id: "selected", x: 120, y: 240 }]);
    expect(cut).toHaveBeenCalledWith({
      nodeIds: ["selected"],
      nodePlacements: [{ id: "selected", x: 120, y: 240 }],
      relationIds: [],
      document: { ...document, nodes: [] },
    });
    expect(paste).toHaveBeenCalledWith({ x: 55, y: 89 });
    expect(historyEntry).toHaveBeenCalledOnce();
  });

  it("commits node and semantic relation deletion through one host transaction", async () => {
    const setSelection = vi.fn<(ids: readonly string[]) => void>();
    const deleteSelection = vi.fn<CanvasSurfaceEvents["onDeleteSelection"]>(async (request) => ({
      document: request.document,
      revision: 2,
      history,
    }));
    const historyTransaction = vi.fn<CanvasSurfaceEvents["onHistoryTransaction"]>(async (action) => action());
    const port = createKonvaEditorPort({
      cancelInteraction() {},
      runFrameMutation: async (mutation) => {
        await mutation();
      },
      host: () => documentElement(),
      layouts: () => document.nodes,
      model: new CanvasDocumentModel(document, {
        documentChanged() {},
        historyEntry() {},
        viewportChanged() {},
      }),
      nodeById: () => undefined,
      nudgeSelection: async () => {},
      nodes: () => [],
      relations: () => [],
      pastePoint: () => ({ x: 0, y: 0 }),
      relationIds: () => new Set(["relation-1"]),
      relationBounds: () => new Map(),
      selectableNodeIds: () => ["selected", "visible-offscreen"],
      selectedNodeIds: () => new Set(["selected"]),
      setSelection,
      setViewport() {},
      size: () => ({ width: 800, height: 600 }),
      viewport: () => document.viewport,
      ...hostEvents(),
      onDeleteSelection: deleteSelection,
      onHistoryTransaction: historyTransaction,
    });

    expect(port.getSelectionSnapshot()).toEqual({
      nodeIds: ["selected"],
      nodePlacements: [{ id: "selected", x: 120, y: 240 }],
      relationIds: ["relation-1"],
      bounds: { x: 120, y: 240, width: 300, height: 220 },
    });
    await port.deleteSelection();

    expect(historyTransaction).toHaveBeenCalledOnce();
    expect(deleteSelection).toHaveBeenCalledWith({
      nodeIds: ["selected"],
      relationIds: ["relation-1"],
      document: { ...document, nodes: [] },
    });
    expect(setSelection).toHaveBeenCalledWith([]);
    port.selectAll();
    expect(setSelection).toHaveBeenCalledWith(["selected", "visible-offscreen"]);
  });
});
