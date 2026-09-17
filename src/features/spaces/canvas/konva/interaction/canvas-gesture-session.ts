import type Konva from "konva";
import { shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasPersistenceHistory, CanvasSurfaceEvents } from "../../contract/canvas-editor";
import type { CanvasPoint } from "../../contract/canvas-editor-port";
import type { CanvasDocumentModel } from "../document/canvas-document-model";
import {
  applyCanvasGestureUpdates,
  captureCanvasGestureLayouts,
} from "../document/canvas-document-mutations";
import { sectionAssignments } from "../scene/section-membership";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import type { CanvasGestureFeedback } from "./canvas-snapping";
import { NodeGestureController } from "./node-gesture-controller";

interface CanvasGestureSessionOptions {
  document: CanvasDocumentModel;
  events: CanvasSurfaceEvents;
  layouts(): readonly CanvasNodeLayoutV1[];
  nodes(): readonly SpaceCanvasNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void;
  sectionNodeIds(): ReadonlySet<string>;
  selectedNodeIds(): ReadonlySet<string>;
  stage(): Konva.Stage | undefined;
  zoom(): number;
}

export type CanvasDirectManipulation = "drag" | "transform";
type CompletedCanvasGesture = { kind: "drag"; nodeId: string } | { kind: "transform" };
export function createCanvasGestureSession(options: CanvasGestureSessionOptions) {
  let mutationTail = Promise.resolve();
  let pendingMutations = 0;
  const busy = shallowRef(false);
  const directManipulation = shallowRef<CanvasDirectManipulation>();
  const feedback = shallowRef<CanvasGestureFeedback>();
  const controller = new NodeGestureController({
    layouts: options.layouts,
    nodes: options.nodes,
    previewFrames: options.previewFrames,
    selectedNodeIds: options.selectedNodeIds,
    stage: options.stage,
    updateFeedback: (next) => (feedback.value = next),
    zoom: options.zoom,
  });

  function beginDirectManipulation(kind: CanvasDirectManipulation) {
    directManipulation.value = kind;
    if (kind === "transform") controller.beginTransform();
  }
  async function finish(gesture: CompletedCanvasGesture) {
    const { kind } = gesture;
    try {
      const updates =
        gesture.kind === "drag" ? await controller.endDrag(gesture.nodeId) : await controller.endTransform();
      await enqueue(() => commit(updates, kind));
    } catch (error) {
      finishDirectManipulation(kind);
      throw error;
    }
  }

  function nudge(offset: CanvasPoint) {
    const targetNodeIds = movableSelectionNodeIds();
    return enqueue(() => commitNudge(offset, targetNodeIds));
  }

  function runFrameMutation(
    calculate: () => readonly CanvasNodeFrameUpdate[] | Promise<readonly CanvasNodeFrameUpdate[]>,
  ) {
    return enqueue(async () => commit(await calculate()));
  }

  async function commitNudge(offset: CanvasPoint, targetNodeIds: ReadonlySet<string>) {
    const updates = options
      .layouts()
      .flatMap<CanvasNodeFrameUpdate>(({ nodeId, frame }) =>
        targetNodeIds.has(nodeId)
          ? [{ nodeId, frame: { ...frame, x: frame.x + offset.x, y: frame.y + offset.y } }]
          : [],
      );
    await commit(updates);
  }

  function movableSelectionNodeIds() {
    const nodes = options.nodes();
    const selected = new Set(options.selectedNodeIds());
    for (const node of nodes) {
      if (node.data.sectionId && selected.has(node.data.sectionId)) selected.add(node.id);
    }
    const locked = new Set(nodes.filter(({ data }) => data.locked).map(({ id }) => id));
    return new Set([...selected].filter((id) => !locked.has(id)));
  }

  async function commit(updates: readonly CanvasGestureUpdate[], manipulation?: CanvasDirectManipulation) {
    if (!updates.length) {
      options.previewFrames([]);
      finishDirectManipulation(manipulation);
      return;
    }
    const before = options.document.document;
    const after = applyCanvasGestureUpdates(before, updates);
    const assignments = sectionAssignments(
      updates.map(({ nodeId: movedNodeId }) => movedNodeId),
      after.nodes,
      options.sectionNodeIds(),
      options.nodes(),
    );
    const beforeFrames = captureCanvasGestureLayouts(before, updates);
    options.document.applyLocalDocument(after);
    options.previewFrames([]);
    finishDirectManipulation(manipulation);
    try {
      const committed = await options.events.onCommitGesture({ assignments, document: after });
      options.document.applyLocalDocument(committed.document);
      const afterFrames = captureCanvasGestureLayouts(committed.document, updates);
      controller.restoreFrames(afterFrames);
      options.events.onHistoryEntry({
        undo: () => replay(beforeFrames, committed.history.undo),
        redo: () => replay(afterFrames, committed.history.redo),
      });
    } catch (error) {
      options.document.applyLocalDocument(before);
      controller.restoreFrames(beforeFrames);
      throw error;
    }
  }

  function finishDirectManipulation(kind?: CanvasDirectManipulation) {
    if (directManipulation.value === kind) directManipulation.value = undefined;
  }

  function cancelActiveGesture() {
    const cancelled = controller.cancelActiveGesture();
    directManipulation.value = undefined;
    return cancelled;
  }

  function enqueue<T>(mutation: () => Promise<T>) {
    pendingMutations += 1;
    busy.value = true;
    if (pendingMutations === 1) options.events.onMutationPendingChange(true);
    const result = mutationTail.then(mutation);
    mutationTail = result.then(
      () => undefined,
      () => undefined,
    );
    return result.finally(() => {
      pendingMutations -= 1;
      busy.value = pendingMutations > 0;
      if (pendingMutations === 0) options.events.onMutationPendingChange(false);
    });
  }

  async function replay(
    updates: readonly CanvasGestureUpdate[],
    transition: CanvasPersistenceHistory["undo"],
  ) {
    const document = applyCanvasGestureUpdates(options.document.document, updates);
    const result = await options.events.onHistoryTransition(document, transition);
    options.document.applyLocalDocument(result.document);
  }

  return {
    busy,
    beginDirectManipulation,
    controller,
    directManipulation,
    feedback,
    finish,
    nudge,
    runFrameMutation,
    cancelActiveGesture,
  };
}
