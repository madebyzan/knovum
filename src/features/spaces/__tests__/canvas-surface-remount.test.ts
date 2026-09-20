import { enableAutoUnmount, flushPromises, mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";

import type {
  CanvasJsonImportCommitResult,
  CanvasJsonImportRequest,
} from "../canvas-exchange/json-canvas-import-contract";
import type { SpaceEditorDocument } from "../canvas/contract/canvas-document";
import type { CanvasHostEvents } from "../canvas/contract/canvas-editor";
import { createCanvasDocument } from "../canvas/document/canvas-document-sync";
import { canvasNote, deferred } from "../testing/canvas-fixtures";

const repository = vi.hoisted(() => ({
  load: vi.fn<(spaceId: string) => Promise<{ document: SpaceEditorDocument; revision: number }>>(),
  save: vi.fn<(request: unknown) => Promise<number>>(),
}));
vi.mock("../canvas/persistence/canvas-state-repository", () => ({ canvasStateRepository: repository }));
vi.mock("../canvas/konva/KonvaCanvasSurface.vue", () => ({
  default: {
    name: "KonvaCanvasSurface",
    methods: { flushViewport() {} },
    props: ["document", "events"],
    template: '<div data-testid="surface">{{ document.nodes[0]?.nodeId }}</div>',
  },
}));

import CanvasSurfaceHost from "../canvas/host/CanvasSurfaceHost.vue";

enableAutoUnmount(afterEach);
afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
});

describe("CanvasSurfaceHost remount persistence", () => {
  it("persists reconciliation instead of leaving it as an in-memory repair", async () => {
    vi.useFakeTimers();
    const first = canvasNote("a");
    repository.load.mockResolvedValue({ document: createCanvasDocument([first]), revision: 4 });
    repository.save.mockResolvedValue(5);

    mountHost("space", [first, canvasNote("b")]);
    await flushPromises();
    await vi.advanceTimersByTimeAsync(301);

    expect(repository.save).toHaveBeenCalledWith({
      spaceId: "space",
      baseRevision: 4,
      document: expect.objectContaining({
        nodes: expect.arrayContaining([expect.objectContaining({ nodeId: "b" })]),
      }),
    });
  });

  it("waits for the previous A flush across a keyed A to B to A remount", async () => {
    const pendingSave = deferred<number>();
    repository.load.mockImplementation(async (spaceId) => ({
      document: createCanvasDocument([canvasNote(spaceId)]),
      revision: 1,
    }));
    repository.save.mockReturnValue(pendingSave.promise);
    const first = mountHost("a", [canvasNote("a"), canvasNote("a-2")]);
    await flushPromises();

    first.unmount();
    const middle = mountHost("b", [canvasNote("b")]);
    await flushPromises();
    expect(middle.get('[data-testid="surface"]').text()).toBe("b");
    middle.unmount();
    const second = mountHost("a", [canvasNote("a"), canvasNote("a-2")]);
    await flushPromises();

    expect(repository.save).toHaveBeenCalledOnce();
    expect(repository.load.mock.calls.map(([spaceId]) => spaceId)).toEqual(["a", "b"]);
    expect(second.find('[data-testid="surface"]').exists()).toBe(false);

    pendingSave.resolve(2);
    await flushPromises();

    expect(repository.load.mock.calls.map(([spaceId]) => spaceId)).toEqual(["a", "b", "a"]);
    expect(second.get('[data-testid="surface"]').text()).toBe("a");
  });

  it("waits for an in-flight formal commit before remounting the same space", async () => {
    const pendingCommit = deferred<{
      document: SpaceEditorDocument;
      revision: number;
      history: { undo: () => Promise<{ revision: number }>; redo: () => Promise<{ revision: number }> };
    }>();
    repository.load.mockResolvedValue({ document: createCanvasDocument([canvasNote("a")]), revision: 1 });
    const deleteSelection = vi.fn<() => typeof pendingCommit.promise>(() => pendingCommit.promise);
    const events = {
      onDeleteSelection: deleteSelection,
      onError: vi.fn<(error: unknown) => void>(),
    } as unknown as CanvasHostEvents;
    const first = mountHost("a", [canvasNote("a")], events);
    await flushPromises();
    const surfaceEvents = first.findComponent({ name: "KonvaCanvasSurface" }).props("events") as {
      onDeleteSelection(request: {
        nodeIds: readonly string[];
        relationIds: readonly string[];
        document: SpaceEditorDocument;
      }): Promise<unknown>;
    };
    const commit = surfaceEvents.onDeleteSelection({
      nodeIds: [],
      relationIds: [],
      document: createCanvasDocument([canvasNote("a")]),
    });
    const commitResult = commit.then(
      () => undefined,
      (error: unknown) => error,
    );
    await vi.waitFor(() => expect(deleteSelection).toHaveBeenCalledOnce());
    first.unmount();
    const second = mountHost("a", [canvasNote("a")], events);
    await flushPromises();
    expect(repository.load).toHaveBeenCalledOnce();
    expect(second.find('[data-testid="surface"]').exists()).toBe(false);

    pendingCommit.resolve({
      document: createCanvasDocument([canvasNote("a")]),
      revision: 2,
      history: { undo: async () => ({ revision: 3 }), redo: async () => ({ revision: 4 }) },
    });
    await expect(commitResult).resolves.toEqual(new Error("Canvas transaction target changed"));
    await flushPromises();
    expect(repository.load).toHaveBeenCalledTimes(2);
    expect(second.get('[data-testid="surface"]').text()).toBe("a");
  });
});

function jsonImporter() {
  return vi.fn<(request: CanvasJsonImportRequest) => Promise<CanvasJsonImportCommitResult>>();
}

function mountHost(
  spaceId: string,
  nodes: ReturnType<typeof canvasNote>[],
  events = { onError: vi.fn<(error: unknown) => void>() } as unknown as CanvasHostEvents,
) {
  return mount(CanvasSurfaceHost, {
    props: {
      events,
      importJsonCanvas: jsonImporter(),
      focusNodeIds: [],
      tool: "select",
      nodes,
      relations: [],
      spaceId,
    },
  });
}
