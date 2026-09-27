import type Konva from "konva";
import { shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import { sectionDropMoveCommand } from "../../../structure/section-drop-reparent";
import type { CanvasSurfaceEvents } from "../../contract/canvas-editor";
import type { CanvasPoint } from "../../contract/canvas-editor-port";
import { commandForCanvasGesture } from "../../document/structure-gesture-command";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasDocumentModel } from "../document/canvas-document-model";
import {
  applyCanvasGestureUpdates,
  captureCanvasGestureLayouts,
} from "../document/canvas-document-mutations";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import { movableCanvasSelectionNodeIds } from "./canvas-node-drag-state";
import type { CanvasGestureFeedback } from "./canvas-snapping";
import { NodeGestureController } from "./node-gesture-controller";

interface CanvasGestureSessionOptions {
  document: CanvasDocumentModel;
  events: CanvasSurfaceEvents;
  layouts(): readonly CanvasNodeLayoutV1[];
  nodes(): readonly SpaceCanvasNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void;
  lockedNodeIds(): ReadonlySet<string>;
  selectedNodeIds(): ReadonlySet<string>;
  snapshot(): SpaceEditSnapshot;
  temporarilyExpandedSectionId(): string | undefined;
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
    lockedNodeIds: options.lockedNodeIds,
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
    const targetNodeIds = movableCanvasSelectionNodeIds(options.selectedNodeIds(), options.lockedNodeIds());
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

  async function commit(updates: readonly CanvasGestureUpdate[], manipulation?: CanvasDirectManipulation) {
    if (!updates.length) {
      options.previewFrames([]);
      finishDirectManipulation(manipulation);
      return;
    }
    const before = options.document.document;
    const after = applyCanvasGestureUpdates(before, updates);
    const beforeFrames = captureCanvasGestureLayouts(before, updates);
    options.document.applyLocalDocument(after);
    const expandedSectionId = options.temporarilyExpandedSectionId();
    options.previewFrames([]);
    finishDirectManipulation(manipulation);
    try {
      const command = commandForCanvasGesture(options.snapshot(), before, after);
      const committed =
        command?.kind === "structure.move"
          ? sectionDropMoveCommand(options.snapshot(), command.rootIds, command.delta, expandedSectionId)
          : command;
      if (committed) await options.events.onExecuteSpaceEdit(committed);
      controller.restoreFrames(captureCanvasGestureLayouts(after, updates));
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
    const gestureCommit = mutationTail.then(mutation);
    mutationTail = gestureCommit.then(
      () => undefined,
      () => undefined,
    );
    return gestureCommit.finally(() => {
      pendingMutations -= 1;
      busy.value = pendingMutations > 0;
      if (pendingMutations === 0) options.events.onMutationPendingChange(false);
    });
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
