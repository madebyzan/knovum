import { describe, expect, it, vi } from "vitest";

import type { SpaceEditorDocument } from "../canvas/contract/canvas-document";
import type {
  CanvasHistoryEntry,
  CanvasPersistenceHistory,
  CanvasSurfaceEvents,
} from "../canvas/contract/canvas-editor";
import { createCanvasDocument } from "../canvas/document/canvas-document-sync";
import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
import { createCanvasGestureSession } from "../canvas/konva/interaction/canvas-gesture-session";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

describe("canvas section gestures", () => {
  it("moves section members and preserves a newer viewport on undo", async () => {
    const nodes = [section(), note("member", 40, 40, "section")];
    const entries: CanvasHistoryEntry[] = [];
    const model = new CanvasDocumentModel(createCanvasDocument(nodes), {
      documentChanged() {},
      historyEntry: (entry) => entries.push(entry),
      viewportChanged() {},
    });
    const history: CanvasPersistenceHistory = {
      undo: async () => ({ revision: 2 }),
      redo: async () => ({ revision: 3 }),
    };
    const events = {
      onCommitGesture: vi.fn<CanvasSurfaceEvents["onCommitGesture"]>(async (request) => ({
        document: request.document,
        revision: 1,
        history,
      })),
      onHistoryEntry: (entry: CanvasHistoryEntry) => entries.push(entry),
      onMutationPendingChange() {},
      onDirectManipulationChange() {},
      onHistoryTransition: async (
        document: SpaceEditorDocument,
        transition: CanvasPersistenceHistory["undo"],
      ) => ({ ...(await transition({ baseRevision: 1, document })), document }),
    } as unknown as CanvasSurfaceEvents;
    const session = createCanvasGestureSession({
      document: model,
      events,
      layouts: () => model.document.nodes,
      nodes: () => nodes,
      previewFrames() {},
      sectionNodeIds: () => new Set(["section"]),
      selectedNodeIds: () => new Set(["section"]),
      stage: () => undefined,
      zoom: () => 1,
    });

    await session.nudge({ x: 20, y: 10 });
    expect(model.document.nodes.map(({ frame }) => [frame.x, frame.y])).toEqual([
      [20, 10],
      [60, 50],
    ]);
    model.setViewport({ x: 90, y: 120, zoom: 2 });
    await entries[0]!.undo();

    expect(model.document.viewport).toEqual({ x: 90, y: 120, zoom: 2 });
    expect(model.document.nodes.map(({ frame }) => [frame.x, frame.y])).toEqual([
      [0, 0],
      [40, 40],
    ]);
  });
});

function section(): SpaceCanvasNode {
  return {
    id: "section",
    type: "section",
    data: { kind: "section", title: "Section" },
    initialPosition: { x: 0, y: 0 },
  };
}

function note(id: string, x: number, y: number, sectionId: string): SpaceCanvasNode {
  return {
    id,
    type: "note",
    data: { ...createSpaceNotePayload(id), sectionId },
    initialPosition: { x, y },
  };
}
