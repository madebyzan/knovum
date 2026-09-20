import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";

import type { CanvasJsonImportRequest } from "../canvas-exchange/json-canvas-import-contract";
import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import { createCanvasSelectionActions } from "../canvas/canvas-selection-actions";
import { createJsonCanvasImportActions } from "../canvas-exchange/json-canvas-import-actions";
import type { JsonCanvasNodePreparation } from "../canvas-exchange/json-canvas-node-preparation";
import type { SpaceState } from "../session/session-state";
import type { useSpaceStore } from "../session/space-store";
import { deferred, emptyCanvasDocument, jsonCanvasTextImportRequest } from "../testing/canvas-fixtures";

const persistence = vi.hoisted(() => ({
  deleteSelection: vi.fn<(request: unknown) => Promise<{ revision: number }>>(),
  restoreSelection: vi.fn<(request: unknown) => Promise<{ revision: number }>>(),
  setImportedNodesDeleted: vi.fn<(request: unknown) => Promise<{ revision: number }>>(),
  importJsonCanvas:
    vi.fn<
      (
        spaceId: string,
        nodes: readonly unknown[],
        baseRevision: number,
        document: CanvasDocumentV1,
      ) => Promise<{ importedNodes: readonly { sourceId: string; nodeId: string }[]; revision: number }>
    >(),
}));
const preparation = vi.hoisted(() => ({
  complete: vi.fn<(value: unknown) => void>(),
  discard: vi.fn<(value: JsonCanvasNodePreparation, error: unknown) => Promise<never>>(),
  prepare: vi.fn<(request: CanvasJsonImportRequest) => Promise<JsonCanvasNodePreparation>>(),
}));

vi.mock("../canvas/persistence/canvas-selection-repository", () => ({
  deleteCanvasSelection: persistence.deleteSelection,
  restoreCanvasSelection: persistence.restoreSelection,
  setImportedCanvasNodesDeleted: persistence.setImportedNodesDeleted,
}));
vi.mock("../canvas-exchange/json-canvas-import-repository", () => ({
  importJsonCanvas: persistence.importJsonCanvas,
}));
vi.mock("../canvas-exchange/json-canvas-node-preparation", () => ({
  completeJsonCanvasPreparation: preparation.complete,
  discardJsonCanvasPreparation: preparation.discard,
  prepareJsonCanvasNodes: preparation.prepare,
}));

describe("canvas session commit identity", () => {
  beforeEach(() => vi.clearAllMocks());

  it("does not append an imported node after the active Space changes", async () => {
    const pending = deferred<{
      importedNodes: readonly { sourceId: string; nodeId: string }[];
      revision: number;
    }>();
    persistence.importJsonCanvas.mockReturnValue(pending.promise);
    const activeSpaceId = ref<string | undefined>("a");
    const nodes = ref([]);
    const commitPendingItems = vi.fn<() => { undo(): Promise<void>; redo(): Promise<void> }>(() => ({
      undo: async () => {},
      redo: async () => {},
    }));
    const requestReveal = vi.fn<(ids: readonly string[]) => void>();
    preparation.prepare.mockResolvedValue({
      nodes: [
        {
          sourceId: "text",
          node: {
            id: "node",
            type: "note",
            data: createSpaceNotePayload("Evidence", "Evidence"),
            initialPosition: { x: 0, y: 0 },
          },
        },
      ],
      assetPaths: [],
      backgroundPaths: new Map(),
      taskIds: [],
    });
    const actions = createJsonCanvasImportActions({
      ensureActiveSpacePersisted: async () => {},
      mutations: { commitPendingItems } as never,
      placement: { requestReveal } as never,
      state: { activeSpaceId, nodes } as unknown as SpaceState,
    });
    const transaction = actions.importJsonCanvas(jsonCanvasTextImportRequest());
    await vi.waitFor(() => expect(persistence.importJsonCanvas).toHaveBeenCalledOnce());

    activeSpaceId.value = "b";
    pending.resolve({ importedNodes: [], revision: 1 });

    await expect(transaction).rejects.toThrow("JSON Canvas import target changed");
    expect(nodes.value).toEqual([]);
    expect(commitPendingItems).not.toHaveBeenCalled();
    expect(requestReveal).not.toHaveBeenCalled();
    expect(preparation.complete).toHaveBeenCalledOnce();
  });

  it("does not remove nodes from the newly active Space after deletion completes", async () => {
    const pending = deferred<{ revision: number }>();
    persistence.deleteSelection.mockReturnValue(pending.promise);
    const original = {
      id: "node",
      type: "note" as const,
      data: createSpaceNotePayload("Node", ""),
      initialPosition: { x: 0, y: 0 },
    };
    const store = { activeSpaceId: "a", nodes: [original], relations: [] } as unknown as ReturnType<
      typeof useSpaceStore
    >;
    const transaction = createCanvasSelectionActions(store).deleteSelection({
      nodeIds: ["node"],
      relationIds: [],
      baseRevision: 0,
      document: emptyCanvasDocument,
    });
    await vi.waitFor(() => expect(persistence.deleteSelection).toHaveBeenCalledOnce());

    store.activeSpaceId = "b";
    pending.resolve({ revision: 1 });

    await expect(transaction).rejects.toThrow("Canvas deletion target changed");
    expect(store.nodes).toEqual([original]);
  });
});
