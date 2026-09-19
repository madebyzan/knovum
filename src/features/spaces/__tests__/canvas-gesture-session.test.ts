import { describe, expect, it, vi } from "vitest";

import type { CanvasDocumentV1, CanvasNodeFrameUpdate } from "../canvas/contract/canvas-document";
import type { CanvasPersistenceHistory, CanvasSurfaceEvents } from "../canvas/contract/canvas-editor";
import { createCanvasDocument } from "../canvas/document/canvas-document-sync";
import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
import { createCanvasGestureSession } from "../canvas/konva/interaction/canvas-gesture-session";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

describe("canvas gesture session", () => {
  it("owns the direct-manipulation lifecycle independently from persistence busy state", () => {
    const nodes = [note("member", 40, 40)];
    const { events, model } = harness(nodes);
    const session = gestureSession(model, events, nodes, new Set(["member"]));

    session.beginDirectManipulation("drag");

    expect(session.directManipulation.value).toBe("drag");
    expect(session.busy.value).toBe(false);

    session.cancelActiveGesture();

    expect(session.directManipulation.value).toBeUndefined();
  });

  it("commits keyboard movement without implicitly changing Section membership", async () => {
    const nodes = [section(), note("member", 1_000, 40)];
    const { events, model } = harness(nodes);
    const session = gestureSession(model, events, nodes, new Set(["member"]));

    await session.nudge({ x: -950, y: 0 });

    expect(events.onCommitGesture).toHaveBeenCalledWith(expect.objectContaining({ assignments: [] }));
  });

  it("serializes rapid keyboard nudges from the latest committed document", async () => {
    const nodes = [note("member", 40, 40)];
    const { events, model } = harness(nodes);
    let resolveFirst:
      | ((value: Awaited<ReturnType<CanvasSurfaceEvents["onCommitGesture"]>>) => void)
      | undefined;
    const firstCommit = new Promise<Awaited<ReturnType<CanvasSurfaceEvents["onCommitGesture"]>>>(
      (resolve) => {
        resolveFirst = resolve;
      },
    );
    events.onCommitGesture = vi
      .fn<CanvasSurfaceEvents["onCommitGesture"]>()
      .mockImplementationOnce(() => firstCommit)
      .mockImplementation(async (request) => ({
        document: request.document,
        revision: 2,
        history: { undo: async () => ({ revision: 3 }), redo: async () => ({ revision: 4 }) },
      }));
    const session = gestureSession(model, events, nodes, new Set(["member"]));

    const first = session.nudge({ x: 10, y: 0 });
    const second = session.nudge({ x: 10, y: 0 });
    await Promise.resolve();
    expect(session.busy.value).toBe(true);
    expect(events.onCommitGesture).toHaveBeenCalledTimes(1);
    const firstDocument = vi.mocked(events.onCommitGesture).mock.calls[0]![0].document;
    resolveFirst?.({
      document: firstDocument,
      revision: 1,
      history: { undo: async () => ({ revision: 2 }), redo: async () => ({ revision: 3 }) },
    });
    await first;
    await second;

    expect(session.busy.value).toBe(false);
    expect(events.onCommitGesture).toHaveBeenCalledTimes(2);
    expect(model.document.nodes[0]?.frame.x).toBe(60);
  });

  it("calculates queued frame mutations from the latest committed document", async () => {
    const nodes = [note("member", 40, 40)];
    const { events, model } = harness(nodes);
    const session = gestureSession(model, events, nodes, new Set(["member"]));

    const first = session.runFrameMutation(() => [movedFrame(model, 10)]);
    const second = session.runFrameMutation(() => [movedFrame(model, 10)]);
    await Promise.all([first, second]);

    expect(model.document.nodes[0]?.frame.x).toBe(60);
  });

  it("keeps geometry interaction busy while an asynchronous frame calculation is pending", async () => {
    const nodes = [note("member", 40, 40)];
    const { events, model } = harness(nodes);
    const session = gestureSession(model, events, nodes, new Set(["member"]));
    let resolveCalculation: ((updates: readonly CanvasNodeFrameUpdate[]) => void) | undefined;
    const calculation = new Promise<readonly CanvasNodeFrameUpdate[]>(
      (resolve) => (resolveCalculation = resolve),
    );

    const mutation = session.runFrameMutation(() => calculation);
    await Promise.resolve();
    expect(session.busy.value).toBe(true);
    expect(events.onCommitGesture).not.toHaveBeenCalled();
    resolveCalculation?.([movedFrame(model, 10)]);
    await mutation;

    expect(session.busy.value).toBe(false);
    expect(model.document.nodes[0]?.frame.x).toBe(50);
  });

  it("keeps a queued nudge bound to the selection that received the command", async () => {
    const nodes = [note("a", 0, 0), note("b", 100, 0), note("c", 200, 0)];
    const { events, model } = harness(nodes);
    const commit = events.onCommitGesture;
    let selected = new Set(["a"]);
    let releaseFirst: (() => void) | undefined;
    const firstCommit = new Promise<void>((resolve) => {
      releaseFirst = resolve;
    });
    vi.mocked(events.onCommitGesture).mockImplementationOnce(async (request) => {
      await firstCommit;
      return commit(request);
    });
    const session = gestureSession(model, events, nodes, () => selected);

    const first = session.nudge({ x: 10, y: 0 });
    selected = new Set(["b"]);
    const second = session.nudge({ x: 10, y: 0 });
    selected = new Set(["c"]);
    releaseFirst?.();
    await Promise.all([first, second]);

    expect(model.document.nodes.map(({ frame }) => frame.x)).toEqual([10, 110, 200]);
  });
});

function movedFrame(model: CanvasDocumentModel, offsetX: number) {
  const layout = model.document.nodes[0]!;
  return { nodeId: layout.nodeId, frame: { ...layout.frame, x: layout.frame.x + offsetX } };
}

function harness(nodes: SpaceCanvasNode[]) {
  const model = new CanvasDocumentModel(createCanvasDocument(nodes), {
    documentChanged() {},
    historyEntry() {},
    viewportChanged() {},
  });
  const history = {
    undo: vi.fn<CanvasPersistenceHistory["undo"]>(async () => ({ revision: 2 })),
    redo: vi.fn<CanvasPersistenceHistory["redo"]>(async () => ({ revision: 3 })),
  };
  const events = {
    onCommitGesture: vi.fn<CanvasSurfaceEvents["onCommitGesture"]>(async (request) => ({
      document: request.document,
      revision: 1,
      history,
    })),
    onHistoryEntry() {},
    onMutationPendingChange() {},
    onDirectManipulationChange() {},
    onHistoryTransition: async (
      document: CanvasDocumentV1,
      transition: CanvasPersistenceHistory["undo"],
    ) => ({
      ...(await transition({ baseRevision: 1, document })),
      document,
    }),
  } as unknown as CanvasSurfaceEvents;
  return { events, model };
}

function gestureSession(
  model: CanvasDocumentModel,
  events: CanvasSurfaceEvents,
  nodes: SpaceCanvasNode[],
  selected: ReadonlySet<string> | (() => ReadonlySet<string>),
) {
  return createCanvasGestureSession({
    document: model,
    events,
    layouts: () => model.document.nodes,
    nodes: () => nodes,
    previewFrames() {},
    sectionNodeIds: () => new Set(["section"]),
    selectedNodeIds: () => (typeof selected === "function" ? selected() : selected),
    stage: () => undefined,
    zoom: () => 1,
  });
}

function section(): SpaceCanvasNode {
  return {
    id: "section",
    type: "section",
    data: { kind: "section", title: "Section" },
    initialPosition: { x: 0, y: 0 },
  };
}

function note(id: string, x: number, y: number, sectionId?: string): SpaceCanvasNode {
  return {
    id,
    type: "note",
    data: { ...createSpaceNotePayload(id), ...(sectionId ? { sectionId } : {}) },
    initialPosition: { x, y },
  };
}
