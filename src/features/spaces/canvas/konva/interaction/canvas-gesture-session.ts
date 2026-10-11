import type Konva from "konva";
import { nextTick, shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import type { CanvasHostEvents } from "../../contracts/canvas-editor";
import { commandForCanvasGesture } from "../../document/structure-gesture-command";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../../schema/document";
import type { CanvasDocumentModel } from "../document/canvas-document-model";
import {
  applyCanvasGestureUpdates,
  captureCanvasGestureLayouts,
} from "../document/canvas-document-mutations";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import type { CanvasGestureFeedback } from "./canvas-snapping";
import { NodeGestureController } from "./node-gesture-controller";

interface CanvasGestureSessionOptions {
  document: CanvasDocumentModel;
  events: CanvasHostEvents;
  layouts(): readonly CanvasNodeLayoutV1[];
  nodes(): readonly SpaceCanvasNode[];
  previewFrames(updates: readonly CanvasGestureUpdate[]): void;
  selectedNodeIds(): ReadonlySet<string>;
  snapshot(): SpaceEditSnapshot;
  stage(): Konva.Stage | undefined;
  zoom(): number;
}

export type CanvasDirectManipulation = "drag" | "transform";
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

  function beginTransform() {
    directManipulation.value = "transform";
    controller.beginTransform();
  }
  async function finishTransform() {
    try {
      const updates = await controller.endTransform();
      await enqueue(() => commit(updates, "transform"));
    } catch (error) {
      finishDirectManipulation("transform");
      throw error;
    }
  }

  function runFrameMutation(
    calculate: () => readonly CanvasNodeFrameUpdate[] | Promise<readonly CanvasNodeFrameUpdate[]>,
  ) {
    return enqueue(async () => commit(await calculate()));
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
    try {
      const command = commandForCanvasGesture(options.snapshot(), before, after);
      if (command) await options.events.onExecuteSpaceEdit(command);
      controller.restoreFrames(captureCanvasGestureLayouts(after, updates));
    } catch (error) {
      options.document.applyLocalDocument(before);
      controller.restoreFrames(beforeFrames);
      throw error;
    } finally {
      await nextTick();
      options.previewFrames([]);
      finishDirectManipulation(manipulation);
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
    beginTransform,
    controller,
    directManipulation,
    feedback,
    finishTransform,
    runFrameMutation,
    executeCommand: (command: Parameters<CanvasHostEvents["onExecuteSpaceEdit"]>[0]) =>
      enqueue(() => options.events.onExecuteSpaceEdit(command)),
    cancelActiveGesture,
  };
}
