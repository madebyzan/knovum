import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";

import { createJsonCanvasImportActions } from "../../../canvas-exchange/json-canvas-import-actions";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceState } from "../../../session/session-state";
import type { useSpaceStore } from "../../../session/space-store";
import {
  canvasNote,
  emptyCanvasDocument,
  jsonCanvasTextImportRequest,
} from "../../../testing/canvas-fixtures";
import { createCanvasSelectionActions } from "../../canvas-selection-actions";

const persistence = vi.hoisted(() => ({
  deleteSelection: vi.fn<(request: unknown) => Promise<{ revision: number }>>(),
  restoreSelection: vi.fn<(request: unknown) => Promise<{ revision: number }>>(),
  setImportedNodesDeleted: vi.fn<(request: unknown) => Promise<{ revision: number }>>(),
  importJsonCanvas:
    vi.fn<() => Promise<{ importedNodes: { sourceId: string; nodeId: string }[]; revision: number }>>(),
}));
const clipboard = vi.hoisted(() => ({ write: vi.fn<() => Promise<boolean>>() }));
const preparation = vi.hoisted(() => ({ prepare: vi.fn<() => Promise<unknown>>() }));

vi.mock("../canvas-selection-repository", () => ({
  deleteCanvasSelection: persistence.deleteSelection,
  restoreCanvasSelection: persistence.restoreSelection,
  setImportedCanvasNodesDeleted: persistence.setImportedNodesDeleted,
}));
vi.mock("../../../canvas-exchange/json-canvas-import-repository", () => ({
  importJsonCanvas: persistence.importJsonCanvas,
}));
vi.mock("../../../clipboard/transfer", () => ({ writeSpaceClipboardDirect: clipboard.write }));
vi.mock("../../../knowledge/indexing/index-scheduler", () => ({
  scheduleKnowledgeIndex: vi.fn<() => void>(),
}));
vi.mock("../../../canvas-exchange/json-canvas-node-preparation", () => ({
  completeJsonCanvasPreparation: vi.fn<() => void>(),
  discardJsonCanvasPreparation: vi.fn<() => void>(),
  prepareJsonCanvasNodes: preparation.prepare,
}));

describe("canvas persistence history", () => {
  beforeEach(() => vi.clearAllMocks());

  it("writes the clipboard before one atomic cut command", async () => {
    clipboard.write.mockResolvedValue(true);
    persistence.deleteSelection.mockResolvedValue({ revision: 1 });
    const original = canvasNote("node");
    const store = testStore([original]);

    await createCanvasSelectionActions(store).cutSelection({
      nodeIds: ["node"],
      nodePlacements: [{ id: "node", x: 12, y: 24 }],
      relationIds: [],
      baseRevision: 0,
      document: emptyCanvasDocument,
    });

    expect(clipboard.write).toHaveBeenCalledWith([{ node: original, position: { x: 12, y: 24 } }]);
    expect(clipboard.write.mock.invocationCallOrder[0]).toBeLessThan(
      persistence.deleteSelection.mock.invocationCallOrder[0] ?? 0,
    );
    expect(persistence.deleteSelection).toHaveBeenCalledWith({
      baseRevision: 0,
      document: emptyCanvasDocument,
      nodeIds: ["node"],
      relationIds: [],
      spaceId: "a",
      survivorChanges: [],
    });
  });

  it("replays delete history with one persistence command per transition", async () => {
    persistence.deleteSelection.mockResolvedValue({ revision: 1 });
    persistence.restoreSelection.mockResolvedValue({ revision: 2 });
    const original = canvasNote("node");
    const store = testStore([original]);
    const result = await createCanvasSelectionActions(store).deleteSelection({
      nodeIds: ["node"],
      relationIds: [],
      baseRevision: 0,
      document: emptyCanvasDocument,
    });

    await result.history.undo({ baseRevision: 1, document: emptyCanvasDocument });
    expect(persistence.restoreSelection).toHaveBeenCalledOnce();
    expect(store.nodes).toEqual([original]);
    await result.history.redo({ baseRevision: 2, document: emptyCanvasDocument });
    expect(persistence.deleteSelection).toHaveBeenCalledTimes(2);
    expect(store.nodes).toEqual([]);
  });

  it("replays import history with one persistence command per transition", async () => {
    const imported = canvasNote("node");
    preparation.prepare.mockResolvedValue({
      nodes: [{ sourceId: "text", node: imported }],
      assetPaths: [],
      backgroundPaths: new Map(),
      taskIds: [],
    });
    persistence.importJsonCanvas.mockResolvedValue({
      importedNodes: [{ sourceId: "text", nodeId: "node" }],
      revision: 1,
    });
    persistence.setImportedNodesDeleted.mockResolvedValue({ revision: 2 });
    const activeSpaceId = ref<string | undefined>("a");
    const nodes = ref<SpaceCanvasNode[]>([]);
    const relations = ref([]);
    const actions = createJsonCanvasImportActions({
      ensureActiveSpacePersisted: async () => {},
      mutations: { commitPendingItems: vi.fn<() => void>() } as never,
      placement: { requestReveal: vi.fn<() => void>() } as never,
      state: { activeSpaceId, nodes, relations } as unknown as SpaceState,
    });
    const result = await actions.importJsonCanvas(jsonCanvasTextImportRequest());

    await result.history.undo({ baseRevision: 1, document: emptyCanvasDocument });
    expect(nodes.value).toEqual([]);
    await result.history.redo({ baseRevision: 2, document: emptyCanvasDocument });
    expect(persistence.setImportedNodesDeleted).toHaveBeenCalledTimes(2);
    expect(nodes.value.map(({ id }) => id)).toEqual(["node"]);
  });
});

function testStore(nodes: SpaceCanvasNode[]) {
  return { activeSpaceId: "a", nodes, relations: [] } as unknown as ReturnType<typeof useSpaceStore>;
}
