import { enableAutoUnmount, flushPromises, mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";

import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportRequest,
} from "../canvas-exchange/json-canvas-import-contract";
import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";
import type {
  CanvasDeleteSelectionResult,
  CanvasHostEvents,
  CanvasSurfaceEvents,
} from "../canvas/contract/canvas-editor";
import { createCanvasDocument } from "../canvas/document/canvas-document-sync";
import { canvasNote, deferred } from "../testing/canvas-fixtures";

const repository = vi.hoisted(() => ({
  load: vi.fn<(spaceId: string) => Promise<{ document: CanvasDocumentV1; revision: number }>>(),
  save: vi.fn<(request: unknown) => Promise<number>>(),
}));
vi.mock("../canvas/persistence/canvas-state-repository", () => ({ canvasStateRepository: repository }));
vi.mock("../canvas/konva/KonvaCanvasSurface.vue", () => ({
  default: {
    name: "KonvaCanvasSurface",
    methods: { flushViewport() {} },
    props: ["document", "events"],
    emits: ["documentChange"],
    template: '<div data-testid="surface">{{ document.nodes[0]?.nodeId }}</div>',
  },
}));

import CanvasSurfaceHost from "../canvas/host/CanvasSurfaceHost.vue";

const persistenceHistory = {
  undo: async () => ({ revision: 2 }),
  redo: async () => ({ revision: 3 }),
};

enableAutoUnmount(afterEach);

afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
});

describe("CanvasSurfaceHost", () => {
  it("does not install a stale load after the active space changes", async () => {
    const loads = new Map<string, (state: { document: CanvasDocumentV1; revision: number }) => void>();
    repository.load.mockImplementation(
      (spaceId: string) => new Promise((resolve) => loads.set(spaceId, resolve)),
    );
    const wrapper = mountHost("a", events());
    await flushPromises();
    await wrapper.setProps({ nodes: [canvasNote("b")], spaceId: "b" });
    await flushPromises();

    loads.get("b")?.({ document: createCanvasDocument([canvasNote("b")]), revision: 2 });
    await flushPromises();
    loads.get("a")?.({ document: createCanvasDocument([canvasNote("a")]), revision: 1 });
    await flushPromises();

    expect(wrapper.get('[data-testid="surface"]').text()).toBe("b");
  });

  it("rejects an in-flight commit after the active space changes", async () => {
    repository.load.mockImplementation(async (spaceId: string) => ({
      document: createCanvasDocument([canvasNote(spaceId)]),
      revision: 1,
    }));
    const pending = deferred<CanvasDeleteSelectionResult>();
    const hostEvents = events();
    hostEvents.onDeleteSelection = async () => pending.promise;
    const wrapper = mountHost("a", hostEvents);
    await flushPromises();
    const transaction = deleteEmptySelection(wrapper);

    await wrapper.setProps({ nodes: [canvasNote("b")], spaceId: "b" });
    pending.resolve({
      document: createCanvasDocument([canvasNote("a")]),
      revision: 2,
      history: persistenceHistory,
    });

    await expect(transaction).rejects.toThrow("Canvas transaction target changed");
    await flushPromises();
    expect(wrapper.get('[data-testid="surface"]').text()).toBe("b");
  });

  it("rejects an in-flight commit after unmount", async () => {
    repository.load.mockResolvedValue({ document: createCanvasDocument([canvasNote("a")]), revision: 1 });
    const pending = deferred<CanvasDeleteSelectionResult>();
    const hostEvents = events();
    hostEvents.onDeleteSelection = async () => pending.promise;
    const wrapper = mountHost("a", hostEvents);
    await flushPromises();
    const transaction = deleteEmptySelection(wrapper);

    wrapper.unmount();
    pending.resolve({
      document: createCanvasDocument([canvasNote("a")]),
      revision: 2,
      history: persistenceHistory,
    });

    await expect(transaction).rejects.toThrow("Canvas transaction target changed");
  });

  it("preserves an in-flight viewport change without replacing committed node frames", async () => {
    repository.load.mockResolvedValue({ document: createCanvasDocument([canvasNote("a")]), revision: 1 });
    const pending = deferred<CanvasDeleteSelectionResult>();
    const hostEvents = events();
    hostEvents.onDeleteSelection = async () => pending.promise;
    const wrapper = mountHost("a", hostEvents);
    await flushPromises();
    const transaction = deleteEmptySelection(wrapper);
    await wrapper.setProps({ nodes: [canvasNote("a"), canvasNote("b")] });
    await flushPromises();
    const surface = wrapper.findComponent({ name: "KonvaCanvasSurface" });
    const presented = surface.props("document") as CanvasDocumentV1;
    surface.vm.$emit("documentChange", {
      ...presented,
      nodes: presented.nodes.map((node, index) => (index === 0 ? { ...node, zIndex: 7 } : node)),
      viewport: { x: 80, y: 120, zoom: 1.5 },
    });
    const committed = {
      ...presented,
      nodes: [{ ...presented.nodes[0]!, frame: { ...presented.nodes[0]!.frame, x: 240 } }],
    };
    pending.resolve({ document: committed, revision: 2, history: persistenceHistory });

    await transaction;
    await flushPromises();
    const document = surface.props("document") as CanvasDocumentV1;
    expect(document.nodes.map(({ nodeId }) => nodeId)).toEqual(["a", "b"]);
    expect(document.nodes[0]?.frame.x).toBe(240);
    expect(document.nodes[0]?.zIndex).toBe(7);
    expect(document.viewport).toEqual({ x: 80, y: 120, zoom: 1.5 });
  });
});

function events(): CanvasHostEvents {
  return {
    onCommitGesture: async (request) => ({
      document: request.document,
      revision: request.baseRevision + 1,
      history: persistenceHistory,
    }),
    onCopyNodes: async () => {},
    onCreateRelation: async () => {},
    onCreateTextNode: async () => "created",
    onCutSelection: async (request) => ({
      document: request.document,
      revision: request.baseRevision + 1,
      history: persistenceHistory,
    }),
    onDeleteSelection: async (request) => ({
      document: request.document,
      revision: request.baseRevision + 1,
      history: persistenceHistory,
    }),
    onDuplicateNodes: async () => {},
    onInsertText: async () => {},
    onInsertUrl: async () => {},
    onImportFiles: async () => [],
    onNodeActivate: async () => {},
    onPasteNodes: async () => false,
    onError() {},
    onHistoryEntry() {},
    onHistoryTransaction: async (action) => action(),
    onMutationPendingChange() {},
    onDirectManipulationChange() {},
    onReady() {},
    onSelectionChange() {},
    onViewportChange() {},
  };
}

function jsonImporter() {
  return vi.fn<(request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>>();
}

function mountHost(spaceId: string, hostEvents: CanvasHostEvents) {
  return mount(CanvasSurfaceHost, {
    props: {
      events: hostEvents,
      importJsonCanvas: jsonImporter(),
      focusNodeIds: [],
      tool: "select",
      nodes: [canvasNote(spaceId)],
      relations: [],
      spaceId,
    },
  });
}

function deleteEmptySelection(wrapper: ReturnType<typeof mountHost>) {
  const surfaceEvents = wrapper
    .findComponent({ name: "KonvaCanvasSurface" })
    .props("events") as CanvasSurfaceEvents;
  return surfaceEvents.onDeleteSelection({
    nodeIds: [],
    relationIds: [],
    document: createCanvasDocument([canvasNote("a")]),
  });
}
