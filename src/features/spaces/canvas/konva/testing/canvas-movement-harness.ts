import { vi } from "vitest";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasSurfaceEvents } from "../../contract/canvas-editor";
import { createCanvasDocument } from "../../document/canvas-document-sync";
import { createKonvaEditorPort } from "../create-canvas-editor-port";
import { CanvasDocumentModel } from "../document/canvas-document-model";
import { createCanvasGestureSession } from "../interaction/canvas-gesture-session";
import { editorPortHostEvents } from "./editor-port-fixture";

export function movementHarness(nodes: SpaceCanvasNode[], selection = new Set(["s1", "s2"])) {
  const documentChanged = vi.fn<(document: SpaceEditorDocument) => void>();
  const model = new CanvasDocumentModel(createCanvasDocument(nodes), {
    documentChanged,
    historyEntry() {},
    viewportChanged() {},
  });
  const events = editorPortHostEvents();
  events.onCommitGesture = vi.fn<CanvasSurfaceEvents["onCommitGesture"]>(events.onCommitGesture);
  events.onHistoryEntry = vi.fn<CanvasSurfaceEvents["onHistoryEntry"]>();
  events.onMutationPendingChange = vi.fn<CanvasSurfaceEvents["onMutationPendingChange"]>();
  const session = createCanvasGestureSession({
    document: model,
    events,
    layouts: () => model.document.nodes,
    nodes: () => nodes,
    previewFrames() {},
    selectedNodeIds: () => selection,
    sectionNodeIds: () => new Set(nodes.filter((node) => node.data.kind === "section").map(({ id }) => id)),
    stage: () => undefined,
    zoom: () => 1,
  });
  const port = createKonvaEditorPort({
    ...events,
    model,
    cancelInteraction() {},
    animateViewport() {},
    setViewport() {},
    setSelection() {},
    runFrameMutation: session.runFrameMutation,
    nudgeSelection: session.nudge,
    layouts: () => model.document.nodes,
    nodes: () => nodes,
    nodeById: (id) => nodes.find((node) => node.id === id),
    selectedNodeIds: () => selection,
    selectableNodeIds: () => nodes.map((node) => node.id),
    relationIds: () => new Set(),
    relationBounds: () => new Map(),
    relations: () => [],
    host: () => document.createElement("div"),
    size: () => ({ width: 800, height: 600 }),
    viewport: () => model.document.viewport,
    pastePoint: () => ({ x: 0, y: 0 }),
    palette: () => {
      throw new Error("This fixture does not render a palette");
    },
  });
  return { nodes, selection, model, session, port, events, documentChanged };
}
